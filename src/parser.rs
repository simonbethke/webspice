//! Zeilen-State-Machine für LTspice `.asc`-Dateien.
//!
//! Aktueller Stand (Task 05): `VERSION`, `SHEET`, `WIRE`, `FLAG`, `BUSTAP`,
//! `SYMBOL`, `SYMATTR`, `TEXT`, `LINE`, `RECTANGLE`, `CIRCLE`, `ARC` werden
//! geparst. Task 06 (Hardening & Integration) folgt (s. `tasks/tasklog.md`).

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::error::Error;
use crate::model::{
    Bustap, Flag, GraphicPrimitive, LineStyle, Orientation, Schematic, Sheet, Symbol,
    TextDirective, WindowSpec, Wire,
};

/// Wandelt die Sequenz `(Zeilennummer, Zeile)` in ein [Schematic] um.
///
/// Leere Zeilen werden übersprungen. Unbekannte Keywords (z. B. `IOPIN`,
/// `DATAFLAG`) werden roh (Roundtrip) in `Symbol.extras` gespeichert,
/// wenn ein Symbol offen ist, sonst in `Schematic.extras` — nie ein
/// Fehler (Forward-Kompatibilität, Spezifikation §6.3).
///
/// `SYMATTR`- und `WINDOW`-Zeilen gehören zum zuletzt deklarierten
/// `SYMBOL` (Zustand `current`); nur bekannte Struktur-Keywords
/// (`SYMBOL`, `VERSION`, `SHEET`, `WIRE`, `FLAG`, `BUSTAP`, `TEXT`,
/// `LINE`, `RECTANGLE`, `CIRCLE`, `ARC`) und das Datei-Ende schließen
/// ein offenes Symbol (Flush), siehe `docs/documentation.md` (§ Format-Referenz).
pub(crate) fn parse_lines<I>(lines: I) -> Result<Schematic, Error>
where
    I: IntoIterator<Item = (usize, String)>,
{
    let mut schematic = Schematic::default();
    let mut current: Option<Symbol> = None;
    for (line_no, line) in lines {
        let mut tokens = line.split_whitespace();
        let Some(keyword) = tokens.next() else {
            continue; // leere Zeile
        };

        if keyword == "SYMATTR" {
            let Some(sym) = current.as_mut() else {
                return Err(Error::OrphanedSymAttr { line: line_no });
            };
            let (key, value) = symattr_key_value(line_no, line.as_str())?;
            sym.attributes.insert(key, value);
            continue;
        }

        // Unbekannte Keywords und `WINDOW`: bleiben im Symbol-Scope,
        // schließen das offene Symbol nicht.
        if !is_structural_keyword(keyword) {
            let raw = line.trim_end().to_string();
            if let Some(sym) = current.as_mut() {
                if keyword == "WINDOW" {
                    if let Some(win) = window_spec(line_no, tokens) {
                        sym.windows.push(win);
                        continue;
                    }
                }
                sym.extras.push(raw);
            } else {
                schematic.extras.push(raw);
            }
            continue;
        }

        // Bekannte Struktur-Keywords schließen ein offenes Symbol.
        if let Some(sym) = current.take() {
            schematic.symbols.push(sym);
        }

        match keyword {
            "VERSION" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "VERSION", &args, 1)?;
                schematic.version = Some(args[0].to_string());
            }
            "SHEET" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "SHEET", &args, 3)?;
                let page = int32(line_no, "page", args[0])?;
                let width = int32(line_no, "width", args[1])?;
                let height = int32(line_no, "height", args[2])?;
                if page < 1 {
                    return Err(invalid_number(line_no, "page", args[0]));
                }
                if width < 0 || height < 0 {
                    let (field, tok) = if width < 0 {
                        ("width", args[1])
                    } else {
                        ("height", args[2])
                    };
                    return Err(invalid_number(line_no, field, tok));
                }
                schematic.sheet = Some(Sheet {
                    page,
                    width,
                    height,
                });
            }
            "WIRE" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "WIRE", &args, 4)?;
                schematic.wires.push(Wire {
                    x1: int32(line_no, "x1", args[0])?,
                    y1: int32(line_no, "y1", args[1])?,
                    x2: int32(line_no, "x2", args[2])?,
                    y2: int32(line_no, "y2", args[3])?,
                });
            }
            "FLAG" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "FLAG", &args, 3)?;
                schematic.flags.push(Flag {
                    x: int32(line_no, "x", args[0])?,
                    y: int32(line_no, "y", args[1])?,
                    name: args[2].to_string(),
                });
            }
            "BUSTAP" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "BUSTAP", &args, 2)?;
                schematic.bustaps.push(Bustap {
                    x: int32(line_no, "x", args[0])?,
                    y: int32(line_no, "y", args[1])?,
                });
            }
            "SYMBOL" => {
                let args: Vec<&str> = tokens.collect();
                expect_arg_count(line_no, "SYMBOL", &args, 4)?;
                let orientation =
                    Orientation::from_token(args[3]).ok_or_else(|| Error::InvalidToken {
                        line: line_no,
                        token: "SYMBOL",
                        field: String::from("orientation"),
                        found: args[3].to_string(),
                    })?;
                current = Some(Symbol {
                    name: args[0].to_string(),
                    x: int32(line_no, "x", args[1])?,
                    y: int32(line_no, "y", args[2])?,
                    orientation,
                    attributes: HashMap::new(),
                    windows: Vec::new(),
                    extras: Vec::new(),
                });
            }
            "TEXT" => {
                // Mindestens 5 Argumente (x, y, align, size, content);
                // obendrauf keine Obergrenze — content enthält Leerzeichen.
                let args: Vec<&str> = tokens.collect();
                if args.len() < 5 {
                    let field = match args.len() {
                        0 => "x",
                        1 => "y",
                        2 => "align",
                        _ if args.len() < 4 => "size",
                        _ => "content",
                    };
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "TEXT",
                        field: field.to_string(),
                    });
                }
                let x = int32(line_no, "x", args[0])?;
                let y = int32(line_no, "y", args[1])?;
                if !matches!(args[2], "Left" | "Center" | "Right") {
                    return Err(Error::InvalidToken {
                        line: line_no,
                        token: "TEXT",
                        field: String::from("align"),
                        found: args[2].to_string(),
                    });
                }
                let size = int32(line_no, "size", args[3])?;
                if size < 1 {
                    return Err(invalid_number(line_no, "size", args[3]));
                }
                // Die count-Prüfung oben garantiert, dass es 5 Tokens gibt.
                let text = payload_after(line.as_str(), 5).ok_or_else(|| Error::MissingField {
                    line: line_no,
                    token: "TEXT",
                    field: String::from("content"),
                })?;
                schematic.texts.push(TextDirective {
                    x,
                    y,
                    align: args[2].to_string(),
                    size,
                    is_command: text.starts_with('!'),
                    text: text.to_string(),
                });
            }
            "LINE" => {
                let args: Vec<&str> = tokens.collect();
                let (style, c, flag) = graphic_args(line_no, "LINE", &args, 4)?;
                schematic.graphics.push(GraphicPrimitive::Line {
                    style,
                    x1: c[0],
                    y1: c[1],
                    x2: c[2],
                    y2: c[3],
                    flag,
                });
            }
            "RECTANGLE" => {
                let args: Vec<&str> = tokens.collect();
                let (style, c, flag) = graphic_args(line_no, "RECTANGLE", &args, 4)?;
                schematic.graphics.push(GraphicPrimitive::Rectangle {
                    style,
                    x1: c[0],
                    y1: c[1],
                    x2: c[2],
                    y2: c[3],
                    flag,
                });
            }
            "CIRCLE" => {
                let args: Vec<&str> = tokens.collect();
                let (style, c, flag) = graphic_args(line_no, "CIRCLE", &args, 4)?;
                schematic.graphics.push(GraphicPrimitive::Circle {
                    style,
                    x1: c[0],
                    y1: c[1],
                    x2: c[2],
                    y2: c[3],
                    flag,
                });
            }
            "ARC" => {
                let args: Vec<&str> = tokens.collect();
                let (style, c, flag) = graphic_args(line_no, "ARC", &args, 8)?;
                schematic.graphics.push(GraphicPrimitive::Arc {
                    style,
                    x1: c[0],
                    y1: c[1],
                    x2: c[2],
                    y2: c[3],
                    x3: c[4],
                    y3: c[5],
                    x4: c[6],
                    y4: c[7],
                    flag,
                });
            }
            // Nicht erreichbar: alle nicht-strukturellen Keywords
            // (inkl. `WINDOW`) wurden schon oben roh abgelegt.
            _ => {}
        }
    }
    // Datei-Ende: offenes Symbol flushen.
    if let Some(sym) = current.take() {
        schematic.symbols.push(sym);
    }
    Ok(schematic)
}

/// Zerlegt `SYMATTR <key> <value…>` in Key und Wert.
///
/// Der Wert stammt aus der Originalzeile (vorne/hinten getrimmt),
/// damit innere Leerzeichen erhalten bleiben.
fn symattr_key_value(line_no: usize, line: &str) -> Result<(String, String), Error> {
    // Das Keyword-Match garantiert, dass die Zeile mit `SYMATTR` beginnt
    // (vorherige Whitespace-Trennung ist egal).
    let rest = line
        .trim_start()
        .strip_prefix("SYMATTR")
        .map(str::trim_start)
        .unwrap_or_default();
    let Some(idx) = rest.find(char::is_whitespace) else {
        let field = if rest.is_empty() { "key" } else { "value" };
        return Err(Error::MissingField {
            line: line_no,
            token: "SYMATTR",
            field: String::from(field),
        });
    };
    Ok((rest[..idx].to_string(), rest[idx..].trim().to_string()))
}

/// Liefert den Zeilen-Rest nach den ersten `n` Tokens, vorne und hinten
/// getrimmt; innere Leerzeichen bleiben erhalten.
///
/// Liefert `None`, wenn die Zeile weniger als `n` Tokens enthält.
fn payload_after(line: &str, n: usize) -> Option<&str> {
    let mut rest = line.trim_start();
    for _ in 0..n {
        let token = rest.split(char::is_whitespace).find(|t| !t.is_empty())?;
        let after = &rest[token.len()..];
        if after.is_empty() {
            return None;
        }
        rest = after.trim_start();
    }
    Some(rest.trim_end())
}

/// Erzeugt einen [Error::InvalidNumber] für einen defekten Zahlenwert.
fn invalid_number(line: usize, field: &str, found: &str) -> Error {
    Error::InvalidNumber {
        line,
        field: field.to_string(),
        found: found.to_string(),
    }
}

/// Parst `<style> <Koordinaten…> [flag]` für die grafischen Tokens
/// (`LINE`, `RECTANGLE`, `CIRCLE`, `ARC`) und prüft die Argumentanzahl.
/// Ein optionales numerisches Schluss-Flag (z. B. `2`) ist erlaubt und
/// wird als `Some` zurückgegeben; ein zweites zusätzliches Token ist
/// `UnexpectedToken`, ein nicht-numerisches Flag ist `InvalidNumber`.
fn graphic_args(
    line_no: usize,
    keyword: &'static str,
    args: &[&str],
    n_coords: usize,
) -> Result<(LineStyle, [i32; 8], Option<i32>), Error> {
    let base = 1 + n_coords;
    if args.len() > base + 1 {
        // `args.len() > base + 1` → Index `base + 1` ist gültig.
        return Err(Error::UnexpectedToken {
            line: line_no,
            expected: String::from("end of line"),
            found: args[base + 1].to_string(),
        });
    }
    if args.len() < base {
        return Err(Error::MissingField {
            line: line_no,
            token: keyword,
            field: format!("argument {base}"),
        });
    }
    let style = LineStyle::from_token(args[0]).ok_or_else(|| Error::InvalidToken {
        line: line_no,
        token: keyword,
        field: String::from("style"),
        found: args[0].to_string(),
    })?;
    let mut coords = [0_i32; 8];
    for (i, tok) in args[1..=n_coords].iter().enumerate() {
        let field = match i {
            0 => "x1",
            1 => "y1",
            2 => "x2",
            3 => "y2",
            4 => "x3",
            5 => "y3",
            6 => "x4",
            _ => "y4",
        };
        coords[i] = int32(line_no, field, tok)?;
    }
    let flag = args
        .get(base)
        .map(|tok| int32(line_no, "flag", tok))
        .transpose()?;
    Ok((style, coords, flag))
}

/// Parst ein Koordinaten-/Größen-Argument als `i32`.
///
/// Parse- und Range-Fehler werden zu [Error::InvalidNumber] umgewandelt.
fn int32(line: usize, field: &str, tok: &str) -> Result<i32, Error> {
    tok.parse::<i32>()
        .map_err(|_| invalid_number(line, field, tok))
}

/// Prüft, dass eine Zeile exakt `n` Argumente (nach dem Keyword) hat.
///
/// Zu wenig → [Error::MissingField], zu viel → [Error::UnexpectedToken]
/// (beide mit Zeilennummer).
fn expect_arg_count(
    line: usize,
    token: &'static str,
    args: &[&str],
    n: usize,
) -> Result<(), Error> {
    match args.len().cmp(&n) {
        Ordering::Less => Err(Error::MissingField {
            line,
            token,
            field: format!("argument {n}"),
        }),
        Ordering::Greater => Err(Error::UnexpectedToken {
            line,
            expected: String::from("end of line"),
            found: args.get(n).copied().unwrap_or_default().to_string(),
        }),
        Ordering::Equal => Ok(()),
    }
}

/// Bekannte Struktur-Keywords, die ein offenes Symbol schließen.
///
/// Alle anderen Keywords (z. B. `WINDOW`, `IOPIN`, `DATAFLAG`) bleiben
/// im Symbol-Scope und werden roh (Roundtrip) abgelegt.
fn is_structural_keyword(keyword: &str) -> bool {
    matches!(
        keyword,
        "VERSION"
            | "SHEET"
            | "WIRE"
            | "FLAG"
            | "BUSTAP"
            | "SYMBOL"
            | "TEXT"
            | "LINE"
            | "RECTANGLE"
            | "CIRCLE"
            | "ARC"
    )
}

/// Parst eine `WINDOW`-Zeile im Symbol-Scope
/// (`<index> <x> <y> <name> <visible>`).
///
/// Liefert `None` bei fehlerhafter Struktur (≠ 5 Felder oder
/// nicht-numerische Werte) — die Zeile wird dann roh abgelegt.
fn window_spec<'a>(
    line_no: usize,
    mut tokens: impl Iterator<Item = &'a str>,
) -> Option<WindowSpec> {
    let index = int32(line_no, "index", tokens.next()?).ok()?;
    let x = int32(line_no, "x", tokens.next()?).ok()?;
    let y = int32(line_no, "y", tokens.next()?).ok()?;
    let name = tokens.next()?.to_string();
    let visible = int32(line_no, "visible", tokens.next()?).ok()?;
    if tokens.next().is_some() {
        return None;
    }
    Some(WindowSpec {
        index,
        x,
        y,
        name,
        visible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Erzeugt die `(Zeilennummer, Zeile)`-Paare eines Eingabe-Strings.
    fn lines_of<'a>(input: &'a str) -> impl Iterator<Item = (usize, String)> + 'a {
        input
            .split('\n')
            .enumerate()
            .map(|(i, l)| (i + 1, l.to_string()))
    }

    /// Parst einen String direkt zu einem Schematic (Testkürzel).
    fn parse(input: &str) -> Result<Schematic, Error> {
        parse_lines(lines_of(input))
    }

    /// Zeilennummer eines zeilenbezogenen Fehlerwerts.
    fn err_line(err: &Error) -> usize {
        match err {
            Error::UnexpectedToken { line, .. }
            | Error::MissingField { line, .. }
            | Error::InvalidNumber { line, .. }
            | Error::OrphanedSymAttr { line }
            | Error::InvalidToken { line, .. } => *line,
            _ => panic!("erwarte zeilenbezogenen Fehler, aber {err:?}"),
        }
    }

    /// `VERSION` speichert einfache Versionen und `Major.Minor`.
    #[test]
    fn version_is_stored() {
        assert_eq!(parse("VERSION 4").unwrap().version.as_deref(), Some("4"));
        assert_eq!(
            parse("VERSION 4.1").unwrap().version.as_deref(),
            Some("4.1")
        );
    }

    /// `VERSION` mit mehr als einem Argument → `UnexpectedToken`.
    #[test]
    fn version_rejects_extra_args() {
        let err = parse("VERSION 4 extra").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { .. }));
        assert_eq!(err_line(&err), 1);
    }

    /// `SHEET` füllt die Canvas-Definition.
    #[test]
    fn sheet_is_stored() {
        assert_eq!(
            parse("SHEET 1 880 680").unwrap().sheet,
            Some(Sheet {
                page: 1,
                width: 880,
                height: 680,
            })
        );
    }

    /// `SHEET` mit zu wenigen Argumenten → `MissingField` mit Zeilennummer.
    #[test]
    fn sheet_rejects_missing_args() {
        let err = parse("VERSION 4\nSHEET 1 880").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
        assert_eq!(err_line(&err), 2);
    }

    /// `SHEET`: `page >= 1`, `width`/`height` nicht negativ.
    #[test]
    fn sheet_validates_ranges() {
        assert!(matches!(
            parse("SHEET 0 880 680").unwrap_err(),
            Error::InvalidNumber { .. }
        ));
        assert!(matches!(
            parse("SHEET 1 -880 680").unwrap_err(),
            Error::InvalidNumber { .. }
        ));
        assert!(matches!(
            parse("SHEET 1 880 -680").unwrap_err(),
            Error::InvalidNumber { .. }
        ));
    }

    /// `WIRE` füllt `wires`.
    #[test]
    fn wire_is_stored() {
        assert_eq!(
            parse("WIRE 160 96 224 96").unwrap().wires,
            vec![Wire {
                x1: 160,
                y1: 96,
                x2: 224,
                y2: 96
            }]
        );
    }

    /// `WIRE` mit nicht-numerischer Koordinate → `InvalidNumber`.
    #[test]
    fn wire_rejects_bad_number() {
        let err = parse("WIRE 1 2 abc 4").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { .. }));
        assert_eq!(err_line(&err), 1);
    }

    /// `FLAG` speichert den Namen, auch Ground `"0"`.
    #[test]
    fn flag_is_stored() {
        assert_eq!(
            parse("FLAG 160 96 0").unwrap().flags,
            vec![Flag {
                x: 160,
                y: 96,
                name: "0".to_string()
            }]
        );
    }

    /// `FLAG` mit zu vielen Argumenten → `UnexpectedToken`.
    #[test]
    fn flag_rejects_extra_args() {
        let err = parse("FLAG 160 96 Vin extra").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { .. }));
    }

    /// `BUSTAP` füllt `bustaps`.
    #[test]
    fn bustap_is_stored() {
        assert_eq!(
            parse("BUSTAP 160 96").unwrap().bustaps,
            vec![Bustap { x: 160, y: 96 }]
        );
    }

    /// `BUSTAP` mit zu wenig Argumenten → `MissingField`.
    #[test]
    fn bustap_rejects_missing_arg() {
        let err = parse("BUSTAP 160").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
    }

    /// Gemischte Datei: Meta- und Topologie-Tokens landen in den richtigen
    /// Feldern.
    #[test]
    fn mixed_meta_and_topology() {
        let s = parse(
            "VERSION 4\nSHEET 1 880 680\nWIRE 160 96 224 96\n\
             FLAG 160 96 Vin\nFLAG 224 96 0\nBUSTAP 320 64\n",
        )
        .unwrap();
        assert_eq!(s.version.as_deref(), Some("4"));
        assert_eq!(
            s.sheet,
            Some(Sheet {
                page: 1,
                width: 880,
                height: 680
            })
        );
        assert_eq!(s.wires.len(), 1);
        assert_eq!(s.flags.len(), 2);
        assert_eq!(s.bustaps, vec![Bustap { x: 320, y: 64 }]);
    }

    /// SYMBOL + SYMATTR: Attribute hängen am Symbol, Werte sind verbatim.
    #[test]
    fn symbol_collects_symattr() {
        let s = parse(
            "SYMBOL res 160 96 R0\n\
             SYMATTR InstName R1\n\
             SYMATTR Value 10k\n\
             SYMATTR SpiceLine model=ideal temp=25\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 1);
        let sym = &s.symbols[0];
        assert_eq!(sym.name, "res");
        assert_eq!(sym.x, 160);
        assert_eq!(sym.y, 96);
        assert_eq!(sym.orientation, Orientation::R0);
        assert_eq!(
            sym.attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
        assert_eq!(sym.attributes.get("Value").map(String::as_str), Some("10k"));
        assert_eq!(
            sym.attributes.get("SpiceLine").map(String::as_str),
            Some("model=ideal temp=25")
        );
    }

    /// Zwei aufeinanderfolgende Symbole: Attribute mischen sich nicht.
    #[test]
    fn symbols_do_not_mix_attributes() {
        let s = parse(
            "SYMBOL res 160 96 R0\n\
             SYMATTR InstName R1\n\
             SYMBOL cap 320 96 M180\n\
             SYMATTR Value 100n\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 2);
        assert_eq!(
            s.symbols[0].attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
        assert_eq!(s.symbols[0].attributes.len(), 1);
        assert_eq!(s.symbols[1].name, "cap");
        assert_eq!(s.symbols[1].orientation, Orientation::M180);
        assert_eq!(
            s.symbols[1].attributes.get("Value").map(String::as_str),
            Some("100n")
        );
    }

    /// SYMBOL ohne Attribute → leere Attribut-Map.
    #[test]
    fn symbol_without_attributes() {
        let s = parse("SYMBOL npn 80 40 R90").unwrap();
        assert_eq!(s.symbols.len(), 1);
        assert!(s.symbols[0].attributes.is_empty());
        assert_eq!(s.symbols[0].orientation, Orientation::R90);
    }

    /// SYMBOL als letzte Zeile → EOF-Flush funktioniert.
    #[test]
    fn eof_flushes_open_symbol() {
        let s = parse("SHEET 1 880 680\nSYMBOL res 160 96 R0\nSYMATTR InstName R9\n").unwrap();
        assert_eq!(s.symbols.len(), 1);
        assert_eq!(
            s.symbols[0].attributes.get("InstName").map(String::as_str),
            Some("R9")
        );
    }

    /// `SYMATTR` ohne aktives SYMBOL → `OrphanedSymAttr` mit Zeilennummer.
    #[test]
    fn orphaned_symattr_is_an_error() {
        let err = parse("VERSION 4\nSYMATTR Value 1k").unwrap_err();
        assert!(matches!(err, Error::OrphanedSymAttr { line: 2 }));
    }

    /// `SYMATTR` als erste Zeile → `OrphanedSymAttr` auf Zeile 1.
    #[test]
    fn orphaned_symattr_first_line() {
        let err = parse("SYMATTR Value 1k").unwrap_err();
        assert!(matches!(err, Error::OrphanedSymAttr { line: 1 }));
    }

    /// Bekannte Struktur-Zeilen (WIRE, SYMBOL, …) schließen das
    /// offene Symbol.
    #[test]
    fn non_symattr_lines_close_open_symbol() {
        let s = parse(
            "SYMBOL res 160 96 R0\n\
             SYMATTR InstName R1\n\
             WIRE 1 2 3 4\n\
             SYMBOL cap 320 96 R0\n\
             SYMATTR Value 100n\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 2);
        assert_eq!(s.wires.len(), 1);
    }

    /// SYMATTR-Werte behalten innere Leerzeichen (verbatim).
    #[test]
    fn symattr_value_keeps_inner_whitespace() {
        let s = parse("SYMBOL res 0 0 R0\nSYMATTR SpiceLine   model=ideal   temp=25\n").unwrap();
        let sym = &s.symbols[0];
        assert_eq!(
            sym.attributes.get("SpiceLine").map(String::as_str),
            Some("model=ideal   temp=25")
        );
    }

    /// Duplizierte SYMATTR-Keys: letzter Wert gewinnt.
    #[test]
    fn duplicate_symattr_keys_last_wins() {
        let s = parse("SYMBOL res 0 0 R0\nSYMATTR Value 1k\nSYMATTR Value 2k\n").unwrap();
        let sym = &s.symbols[0];
        assert_eq!(sym.attributes.len(), 1);
        assert_eq!(sym.attributes.get("Value").map(String::as_str), Some("2k"));
    }

    /// SYMBOL mit zu wenigen bzw. zu vielen Nachfolgern → Fehler.
    #[test]
    fn symbol_arg_count_is_enforced() {
        let err = parse("SYMBOL res 160 96").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
        let err = parse("SYMBOL res 160 96 R0 extra").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { .. }));
    }

    /// Unbekannte Orientation → `InvalidToken` mit Zeilennummer.
    #[test]
    fn symbol_invalid_orientation() {
        let err = parse("SYMBOL res 160 96 R45").unwrap_err();
        assert!(matches!(err, Error::InvalidToken { line: 1, .. }));
    }

    /// Alle 8 gültigen Orientation-Tokens parsen.
    #[test]
    fn all_orientations_parse() {
        for tok in ["R0", "R90", "R180", "R270", "M0", "M90", "M180", "M270"] {
            let s = parse(&format!("SYMBOL res 1 2 {tok}")).unwrap();
            assert_eq!(s.symbols.len(), 1);
            assert_eq!(s.symbols[0].name, "res");
        }
    }

    /// `!`-Direktive: `is_command == true`, Text verbatim inkl. Präfix.
    #[test]
    fn text_command_directive() {
        let s = parse("TEXT 160 96 Left 4 !.tran 10m").unwrap();
        assert_eq!(s.texts.len(), 1);
        let text = &s.texts[0];
        assert_eq!(text.x, 160);
        assert_eq!(text.y, 96);
        assert_eq!(text.align, "Left");
        assert_eq!(text.size, 4);
        assert!(text.is_command);
        assert_eq!(text.text, "!.tran 10m");
    }

    /// `;`-Kommentar → keine Kommandozeile.
    #[test]
    fn text_comment_is_not_command() {
        let s = parse("TEXT 0 0 Right 2 ; Anmerkung hier").unwrap();
        let text = &s.texts[0];
        assert!(!text.is_command);
        assert_eq!(text.align, "Right");
        assert_eq!(text.text, "; Anmerkung hier");
    }

    /// Freitext ohne Präfix → keine Kommandozeile.
    #[test]
    fn text_plain_is_not_command() {
        let s = parse("TEXT 0 0 Center 1 some label").unwrap();
        let text = &s.texts[0];
        assert!(!text.is_command);
        assert_eq!(text.align, "Center");
        assert_eq!(text.text, "some label");
    }

    /// Mehrfach-Leerzeichen im Inhalt bleiben erhalten (verbatim).
    #[test]
    fn text_content_keeps_inner_whitespace() {
        let s = parse("TEXT 0 0 Left 1 a   b    c").unwrap();
        assert_eq!(s.texts[0].text, "a   b    c");
    }

    /// `size` bzw. `content` fehlt → `MissingField` mit Zeilennummer.
    #[test]
    fn text_missing_fields() {
        let err = parse("VERSION 4\nTEXT 0 0 Left").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
        assert_eq!(err_line(&err), 2);
        let err = parse("TEXT 0 0 Left 4").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
    }

    /// Ungültiges `align` → `InvalidToken` mit Zeilennummer.
    #[test]
    fn text_invalid_align() {
        let err = parse("TEXT 0 0 top 4 hello").unwrap_err();
        assert!(matches!(err, Error::InvalidToken { line: 1, .. }));
    }

    /// Ungültiges `size` (nicht-numerisch bzw. `< 1`) → `InvalidNumber`.
    #[test]
    fn text_invalid_size() {
        assert!(matches!(
            parse("TEXT 0 0 Left 0 hello").unwrap_err(),
            Error::InvalidNumber { .. }
        ));
        assert!(matches!(
            parse("TEXT 0 0 Left abc hello").unwrap_err(),
            Error::InvalidNumber { .. }
        ));
    }

    /// Grafische Primitives: alle 4 Styles pro Primitive, Felder exakt.
    #[test]
    fn line_all_styles() {
        for (tok, style) in [
            ("Normal", LineStyle::Normal),
            ("Dotted", LineStyle::Dotted),
            ("Dashed", LineStyle::Dashed),
            ("Bold", LineStyle::Bold),
        ] {
            let s = parse(&format!("LINE {tok} 0 0 10 20")).unwrap();
            assert_eq!(
                s.graphics,
                vec![GraphicPrimitive::Line {
                    style,
                    x1: 0,
                    y1: 0,
                    x2: 10,
                    y2: 20,
                    flag: None
                }]
            );
        }
    }

    /// `RECTANGLE` in allen 4 Styles (Koordinaten bleiben unverändert).
    #[test]
    fn rectangle_all_styles() {
        for (tok, style) in [
            ("Normal", LineStyle::Normal),
            ("Dotted", LineStyle::Dotted),
            ("Dashed", LineStyle::Dashed),
            ("Bold", LineStyle::Bold),
        ] {
            let s = parse(&format!("RECTANGLE {tok} 0 0 10 20")).unwrap();
            assert_eq!(
                s.graphics,
                vec![GraphicPrimitive::Rectangle {
                    style,
                    x1: 0,
                    y1: 0,
                    x2: 10,
                    y2: 20,
                    flag: None
                }]
            );
        }
    }

    /// `CIRCLE` in allen 4 Styles (Bounding-Box bleibt unverändert).
    #[test]
    fn circle_all_styles() {
        for (tok, style) in [
            ("Normal", LineStyle::Normal),
            ("Dotted", LineStyle::Dotted),
            ("Dashed", LineStyle::Dashed),
            ("Bold", LineStyle::Bold),
        ] {
            let s = parse(&format!("CIRCLE {tok} 0 0 10 20")).unwrap();
            assert_eq!(
                s.graphics,
                vec![GraphicPrimitive::Circle {
                    style,
                    x1: 0,
                    y1: 0,
                    x2: 10,
                    y2: 20,
                    flag: None
                }]
            );
        }
    }

    /// `ARC` in allen 4 Styles (8 Koordinaten exakt).
    #[test]
    fn arc_all_styles() {
        for (tok, style) in [
            ("Normal", LineStyle::Normal),
            ("Dotted", LineStyle::Dotted),
            ("Dashed", LineStyle::Dashed),
            ("Bold", LineStyle::Bold),
        ] {
            let s = parse(&format!("ARC {tok} 1 2 3 4 5 6 7 8")).unwrap();
            assert_eq!(
                s.graphics,
                vec![GraphicPrimitive::Arc {
                    style,
                    x1: 1,
                    y1: 2,
                    x2: 3,
                    y2: 4,
                    x3: 5,
                    y3: 6,
                    x4: 7,
                    y4: 8,
                    flag: None
                }]
            );
        }
    }

    /// ARC zu wenige Nachfolger → `MissingField`; die 9. Zahl ist das
    /// optionale Schluss-Flag; die 10. Zahl → `UnexpectedToken`.
    #[test]
    fn arc_arg_count_and_flag_are_enforced() {
        let err = parse("VERSION 4\nARC Normal 1 2 3 4 5 6 7").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
        assert_eq!(err_line(&err), 2);
        let s = parse("ARC Normal 1 2 3 4 5 6 7 8 9").unwrap();
        assert_eq!(
            s.graphics,
            vec![GraphicPrimitive::Arc {
                style: LineStyle::Normal,
                x1: 1,
                y1: 2,
                x2: 3,
                y2: 4,
                x3: 5,
                y3: 6,
                x4: 7,
                y4: 8,
                flag: Some(9)
            }]
        );
        let err = parse("ARC Normal 1 2 3 4 5 6 7 8 9 10").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 1, .. }));
    }

    /// Ungültiger Style (`DottedD`) → `InvalidToken` mit Zeilennummer.
    #[test]
    fn graphics_invalid_style() {
        let err = parse("LINE DottedD 0 0 1 1").unwrap_err();
        assert!(matches!(err, Error::InvalidToken { line: 1, .. }));
    }

    /// Negative Koordinaten werden akzeptiert (nur Syntax-Check).
    #[test]
    fn graphics_accept_negative_coords() {
        let s = parse("RECTANGLE Bold -5 -10 -1 -1").unwrap();
        assert_eq!(
            s.graphics,
            vec![GraphicPrimitive::Rectangle {
                style: LineStyle::Bold,
                x1: -5,
                y1: -10,
                x2: -1,
                y2: -1,
                flag: None
            }]
        );
    }

    /// Numerisches Schluss-Flag auf `LINE`/`RECTANGLE`/`CIRCLE` wird als
    /// `flag: Some(..)` behalten (echte LTspice-Dateien schreiben z. B. 2/4).
    #[test]
    fn graphics_trailing_flags_are_kept() {
        let s = parse("LINE Normal 1 2 3 4 2\nRECTANGLE Dashed 0 0 5 5 4\nCIRCLE Bold 0 0 4 4 0")
            .unwrap();
        assert_eq!(
            s.graphics,
            vec![
                GraphicPrimitive::Line {
                    style: LineStyle::Normal,
                    x1: 1,
                    y1: 2,
                    x2: 3,
                    y2: 4,
                    flag: Some(2)
                },
                GraphicPrimitive::Rectangle {
                    style: LineStyle::Dashed,
                    x1: 0,
                    y1: 0,
                    x2: 5,
                    y2: 5,
                    flag: Some(4)
                },
                GraphicPrimitive::Circle {
                    style: LineStyle::Bold,
                    x1: 0,
                    y1: 0,
                    x2: 4,
                    y2: 4,
                    flag: Some(0)
                }
            ]
        );
    }

    /// Nicht-numerisches Schluss-Flag → `InvalidNumber` mit Zeilennummer.
    #[test]
    fn graphics_non_numeric_flag_is_invalid_number() {
        let err = parse("LINE Normal 1 2 3 4 abc").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { line: 1, .. }));
    }

    /// Zwei zusätzliche Schluss-Tokens → `UnexpectedToken`.
    #[test]
    fn graphics_two_trailing_tokens_are_rejected() {
        let err = parse("LINE Normal 1 2 3 4 2 3").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 1, .. }));
    }

    /// Graphic-Zeile nach offenem SYMBOL schließt das Symbol.
    #[test]
    fn graphics_line_closes_open_symbol() {
        let s = parse("SYMBOL res 1 2 R0\nSYMATTR InstName R1\nLINE Normal 0 0 5 5\n").unwrap();
        assert_eq!(s.symbols.len(), 1);
        assert_eq!(
            s.symbols[0].attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
        assert_eq!(s.graphics.len(), 1);
    }

    /// TEXT-Zeile nach offenem SYMBOL schließt das Symbol.
    #[test]
    fn text_line_closes_open_symbol() {
        let s = parse(
            "SYMBOL res 160 96 R0\n\
             SYMATTR InstName R1\n\
             TEXT 0 0 Left 4 caption text\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 1);
        assert_eq!(
            s.symbols[0].attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
        assert_eq!(s.texts.len(), 1);
    }

    /// WINDOW-Zeilen zwischen SYMBOL und SYMATTR schließen das Symbol
    /// nicht und landen typisiert in `Symbol.windows`.
    #[test]
    fn window_lines_between_symbol_and_symattr() {
        let s = parse(
            "SYMBOL res 160 96 R0\n\
             WINDOW 1 60 70 Value 1\n\
             WINDOW 3 60 70 SpiceLine 0\n\
             SYMATTR InstName R1\n\
             SYMATTR Value 10k\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 1);
        let sym = &s.symbols[0];
        assert_eq!(
            sym.windows,
            vec![
                WindowSpec {
                    index: 1,
                    x: 60,
                    y: 70,
                    name: "Value".to_string(),
                    visible: 1
                },
                WindowSpec {
                    index: 3,
                    x: 60,
                    y: 70,
                    name: "SpiceLine".to_string(),
                    visible: 0
                }
            ]
        );
        assert_eq!(
            sym.attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
        assert_eq!(sym.attributes.get("Value").map(String::as_str), Some("10k"));
        assert!(sym.extras.is_empty());
        assert!(s.extras.is_empty());
    }

    /// Unbekannte Tokens zwischen SYMBOL und SYMATTR: kein Fehler,
    /// roh in `Symbol.extras`.
    #[test]
    fn unknown_lines_in_symbol_scope_are_kept() {
        let s = parse(
            "SYMBOL npn 480 64 R0\n\
             IOPIN 16 32 left 255\n\
             DATAFLAG 480 64 2\n\
             SYMATTR InstName Q1\n",
        )
        .unwrap();
        assert_eq!(s.symbols.len(), 1);
        let sym = &s.symbols[0];
        assert_eq!(
            sym.extras,
            vec!["IOPIN 16 32 left 255", "DATAFLAG 480 64 2"]
        );
        assert_eq!(
            sym.attributes.get("InstName").map(String::as_str),
            Some("Q1")
        );
        assert!(s.extras.is_empty());
    }

    /// Unbekannte Zeilen ohne offenes Symbol landen roh in
    /// `Schematic.extras`.
    #[test]
    fn unknown_lines_outside_symbol_scope_are_kept() {
        let s = parse("IOPIN 16 32 left 255\nVERSION 4\nDATAFLAG 480 64 2\n").unwrap();
        assert_eq!(s.extras, vec!["IOPIN 16 32 left 255", "DATAFLAG 480 64 2"]);
        assert!(s.symbols.is_empty());
        assert_eq!(s.version.as_deref(), Some("4"));
    }

    /// Fehlgeschaffene WINDOW im Symbol-Scope (≠ 5 Felder) → roh in
    /// `Symbol.extras`, kein Fehler.
    #[test]
    fn malformed_window_is_kept_raw() {
        let s = parse("SYMBOL res 160 96 R0\nWINDOW 1 2 3\nSYMATTR InstName R1\n").unwrap();
        let sym = &s.symbols[0];
        assert!(sym.windows.is_empty());
        assert_eq!(sym.extras, vec!["WINDOW 1 2 3"]);
        assert_eq!(
            sym.attributes.get("InstName").map(String::as_str),
            Some("R1")
        );
    }

    /// WINDOW außerhalb des Symbol-Scope → roh in `Schematic.extras`.
    #[test]
    fn window_outside_symbol_scope_is_kept_raw() {
        let s = parse("WINDOW 1 60 70 Value 1\nSYMBOL res 160 96 R0\n").unwrap();
        assert_eq!(s.extras, vec!["WINDOW 1 60 70 Value 1"]);
        assert!(s.symbols[0].windows.is_empty());
        assert!(s.symbols[0].extras.is_empty());
    }
}
