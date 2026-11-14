//! Parser für LTspice-`.asy`-Symboldefinitionsdateien (Task 09).
//!
//! Geparst werden `Version`, `SYMBOLTYPE`, `SYMATTR`, `WINDOW`, `LINE`,
//! `RECTANGLE`, `CIRCLE`, `ARC`, `TEXT`, `PIN`, `PINATTR` in [AsySymbol];
//! unbekannte Zeilen (z. B. `DRAW`, `ATTRDEF`, `SYMKEY`, `IOPIN`) werden
//! stillschweigend ignoriert — es gibt **keine** Roundtrip-Variante für
//! `.asy`-Dateien (im Gegensatz zu `.asc`).
//!
//! Vertrag: `docs/documentation.md`, Abschnitt „Symbol-Rendering (.asy)";
//! Format-Basis `ltspice_asy_specification.md`.

use std::collections::HashMap;

use serde::Serialize;

use crate::error::Error;
use crate::model::{GraphicPrimitive, WindowSpec};

/// Ein komplett geparstes LTspice-Symbol (`.asy`-Datei).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AsySymbol {
    /// Versions-String aus der Pflicht-`Version`-Zeile (z. B. `"4"`).
    pub version: String,
    /// Token der `SYMBOLTYPE`-Zeile (`"CELL"`/`"BLOCK"`, verbatim);
    /// `None`, wenn keine `SYMBOLTYPE`-Zeile vorhanden ist.
    #[serde(rename = "symbolType")]
    pub symbol_type: Option<String>,
    /// `SYMATTR`-Key→Wert (Doppelungen: letzter Wert gewinnt).
    pub properties: HashMap<String, String>,
    /// `WINDOW`-Zeilen (Sichtbarkeits-Slots).
    pub windows: Vec<WindowSpec>,
    /// Grafische Primitive (`LINE`, `RECTANGLE`, `CIRCLE`, `ARC`).
    pub graphics: Vec<GraphicPrimitive>,
    /// Textzeilen (`TEXT`).
    pub texts: Vec<AsyText>,
    /// Pin-Definitionen (`PIN`) inkl. gebundener `PINATTR`-Eigenschaften.
    pub pins: Vec<AsyPin>,
}

/// Eine `TEXT`-Zeile aus einer `.asy`-Datei.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AsyText {
    /// Position X.
    pub x: i32,
    /// Position Y.
    pub y: i32,
    /// Ausrichtung verbatim (`Left`, `Center`, `Right`, …);
    /// bei der tolerierten Kurzform leer.
    pub justification: String,
    /// Der vollständige Text bis Zeilenende (verbatim).
    pub text: String,
}

/// Eine `PIN`-Zeile aus einer `.asy`-Datei inkl. `PINATTR`-Eigenschaften.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AsyPin {
    /// Position X.
    pub x: i32,
    /// Position Y.
    pub y: i32,
    /// Ausrichtung verbatim (Korpus nutzt meist `NONE`).
    pub justification: String,
    /// Pin-Offset.
    pub offset: i32,
    /// Optionaler Pin-Name verbatim (z. B. `+`, `-`, `NC+`);
    /// `None`, wenn die `PIN`-Zeile keinen Namen hatte.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// `PINATTR`-Key→Wert (Doppelungen: letzter Wert gewinnt).
    pub properties: HashMap<String, String>,
}

/// Wandelt die Sequenz `(Zeilennummer, Zeile)` in ein [AsySymbol] um.
///
/// Regeln (siehe `docs/documentation.md`, „Symbol-Rendering (.asy)"):
/// - Schlüsselwörter sind case-insensitiv.
/// - Die erste nicht-leere Zeile muss `Version <n>` sein, sonst
///   [Error::MissingVersion] (leere Datei: Zeile 1).
/// - `PINATTR` bindet an den zuletzt definierten `PIN`, sonst
///   [Error::OrphanedPinAttr].
/// - Unbekannte Zeilen werden stillschweigend ignoriert.
pub(crate) fn parse_asy_lines<I>(lines: I) -> Result<AsySymbol, Error>
where
    I: IntoIterator<Item = (usize, String)>,
{
    let mut sym = AsySymbol {
        version: String::new(),
        symbol_type: None,
        properties: HashMap::new(),
        windows: Vec::new(),
        graphics: Vec::new(),
        texts: Vec::new(),
        pins: Vec::new(),
    };
    let mut version_seen = false;
    for (line_no, line) in lines {
        let mut tokens = line.split_whitespace();
        let Some(word) = tokens.next() else {
            continue; // leere Zeile
        };
        if !version_seen {
            // Die erste nicht-leere Zeile muss `Version <n>` sein.
            if !word.eq_ignore_ascii_case("Version") {
                return Err(Error::MissingVersion { line: line_no });
            }
            sym.version = version_arg(line_no, tokens)?;
            version_seen = true;
            continue;
        }
        let key = word.to_ascii_lowercase();
        match key.as_str() {
            "version" => {
                sym.version = version_arg(line_no, tokens)?;
            }
            "symboltype" => {
                let args: Vec<&str> = tokens.collect();
                if args.is_empty() {
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "SYMBOLTYPE",
                        field: String::from("type"),
                    });
                }
                if args.len() > 1 {
                    return Err(Error::UnexpectedToken {
                        line: line_no,
                        expected: String::from("end of line"),
                        found: args[1].to_string(),
                    });
                }
                if !(args[0].eq_ignore_ascii_case("CELL") || args[0].eq_ignore_ascii_case("BLOCK"))
                {
                    return Err(Error::InvalidToken {
                        line: line_no,
                        token: "SYMBOLTYPE",
                        field: String::from("type"),
                        found: args[0].to_string(),
                    });
                }
                sym.symbol_type = Some(args[0].to_string());
            }
            "symattr" => {
                let (key, value) = key_value(line_no, "SYMATTR", &line)?;
                sym.properties.insert(key, value);
            }
            "window" => {
                // Gleiche 5-Argument-Form wie `WINDOW` in `.asc`:
                // `WINDOW <index> <x> <y> <just> <n>` (Korpus: 9311/9312
                // Zeilen exakt 5 Argumente; `just` ∈ Left/Center/Top/
                // Bottom/Right, `n` ∈ 0..=5).
                let args: Vec<&str> = tokens.collect();
                if args.len() < 5 {
                    let field = match args.len() {
                        0 => "index",
                        1 => "x",
                        2 => "y",
                        3 => "justification",
                        _ => "visible",
                    };
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "WINDOW",
                        field: field.to_string(),
                    });
                }
                if args.len() > 5 {
                    return Err(Error::UnexpectedToken {
                        line: line_no,
                        expected: String::from("end of line"),
                        found: args[5].to_string(),
                    });
                }
                sym.windows.push(WindowSpec {
                    index: int32(line_no, "index", args[0])?,
                    x: int32(line_no, "x", args[1])?,
                    y: int32(line_no, "y", args[2])?,
                    name: args[3].to_string(),
                    visible: int32(line_no, "visible", args[4])?,
                });
            }
            "line" | "rectangle" | "circle" | "arc" => {
                let keyword = match key.as_str() {
                    "line" => "LINE",
                    "rectangle" => "RECTANGLE",
                    "circle" => "CIRCLE",
                    _ => "ARC",
                };
                let n_coords = if keyword == "ARC" { 8usize } else { 4 };
                let args: Vec<&str> = tokens.collect();
                let (style, c, width) = graphic_args(line_no, keyword, &args, n_coords)?;
                let primitive = match keyword {
                    "LINE" => GraphicPrimitive::Line {
                        style,
                        x1: c[0],
                        y1: c[1],
                        x2: c[2],
                        y2: c[3],
                        flag: width,
                    },
                    "RECTANGLE" => GraphicPrimitive::Rectangle {
                        style,
                        x1: c[0],
                        y1: c[1],
                        x2: c[2],
                        y2: c[3],
                        flag: width,
                    },
                    "CIRCLE" => GraphicPrimitive::Circle {
                        style,
                        x1: c[0],
                        y1: c[1],
                        x2: c[2],
                        y2: c[3],
                        flag: width,
                    },
                    _ => GraphicPrimitive::Arc {
                        style,
                        x1: c[0],
                        y1: c[1],
                        x2: c[2],
                        y2: c[3],
                        x3: c[4],
                        y3: c[5],
                        x4: c[6],
                        y4: c[7],
                        flag: width,
                    },
                };
                sym.graphics.push(primitive);
            }
            "text" => {
                let args: Vec<&str> = tokens.collect();
                if args.len() < 2 {
                    let field = if args.is_empty() { "x" } else { "y" };
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "TEXT",
                        field: field.to_string(),
                    });
                }
                if args.len() == 2 {
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "TEXT",
                        field: String::from("text"),
                    });
                }
                let x = int32(line_no, "x", args[0])?;
                let y = int32(line_no, "y", args[1])?;
                // Formen: 3 Argumente = tolerierte Kurzform `<x> <y>
                // <text…>`; 4 Argumente = Volle Form, falls das 4. Argument
                // eine Größe (`n`) ist — sonst Kurzform; ≥ 5 Argumente =
                // Volle Form `<x> <y> <just> <n> <text…>`.
                let short = match args.len() {
                    3 => true,
                    4 => args[3].parse::<i32>().is_err(),
                    _ => false,
                };
                if short {
                    let text = payload_after(&line, 3).ok_or_else(|| Error::MissingField {
                        line: line_no,
                        token: "TEXT",
                        field: String::from("text"),
                    })?;
                    sym.texts.push(AsyText {
                        x,
                        y,
                        justification: String::new(),
                        text: text.to_string(),
                    });
                } else {
                    // Volle Form `<x> <y> <just> <n> <text…>`;
                    // die Größe `<n>` wird gelesen, aber nicht abgelegt.
                    let justification = args[2].to_string();
                    int32(line_no, "n", args[3])?;
                    if args.len() < 5 {
                        return Err(Error::MissingField {
                            line: line_no,
                            token: "TEXT",
                            field: String::from("text"),
                        });
                    }
                    let text = payload_after(&line, 5).ok_or_else(|| Error::MissingField {
                        line: line_no,
                        token: "TEXT",
                        field: String::from("text"),
                    })?;
                    sym.texts.push(AsyText {
                        x,
                        y,
                        justification,
                        text: text.to_string(),
                    });
                }
            }
            "pin" => {
                let args: Vec<&str> = tokens.collect();
                if args.len() < 4 {
                    let field = match args.len() {
                        0 => "x",
                        1 => "y",
                        2 => "just",
                        _ => "offset",
                    };
                    return Err(Error::MissingField {
                        line: line_no,
                        token: "PIN",
                        field: field.to_string(),
                    });
                }
                if args.len() > 5 {
                    return Err(Error::UnexpectedToken {
                        line: line_no,
                        expected: String::from("end of line"),
                        found: args[5].to_string(),
                    });
                }
                sym.pins.push(AsyPin {
                    x: int32(line_no, "x", args[0])?,
                    y: int32(line_no, "y", args[1])?,
                    justification: args[2].to_string(),
                    offset: int32(line_no, "offset", args[3])?,
                    name: args.get(4).map(|s| s.to_string()),
                    properties: HashMap::new(),
                });
            }
            "pinattr" => {
                let (key, value) = key_value(line_no, "PINATTR", &line)?;
                let Some(pin) = sym.pins.last_mut() else {
                    return Err(Error::OrphanedPinAttr { line: line_no });
                };
                pin.properties.insert(key, value);
            }
            // Unbekannte Zeilen (DRAW, ATTRDEF, SYMKEY, IOPIN, …) werden
            // stillschweigend ignoriert — keine Roundtrip-Variante für `.asy`.
            _ => {}
        }
    }
    if !version_seen {
        return Err(Error::MissingVersion { line: 1 });
    }
    Ok(sym)
}

/// Liest das Pflicht-Argument einer `Version`-Zeile (exakt 1 Token).
fn version_arg<'a>(
    line_no: usize,
    mut tokens: impl Iterator<Item = &'a str>,
) -> Result<String, Error> {
    let Some(arg) = tokens.next() else {
        return Err(Error::MissingField {
            line: line_no,
            token: "VERSION",
            field: String::from("version"),
        });
    };
    if tokens.next().is_some() {
        return Err(Error::UnexpectedToken {
            line: line_no,
            expected: String::from("end of line"),
            found: arg.to_string(),
        });
    }
    Ok(arg.to_string())
}

/// Zerlegt `<keyword> <key> <value…>` in Key und Wert (beide verbatim;
/// der Wert stammt aus der Originalzeile, damit innere Leerzeichen
/// erhalten bleiben).
fn key_value(line_no: usize, keyword: &'static str, line: &str) -> Result<(String, String), Error> {
    // Das Keyword-Match garantiert, dass die (ge-trimhte) Zeile mit
    // dem Keyword beginnt (case-insensitiv).
    let trimmed = line.trim_start();
    let rest = trimmed
        .get(keyword.len()..)
        .unwrap_or_default()
        .trim_start();
    let Some(idx) = rest.find(char::is_whitespace) else {
        let field = if rest.is_empty() { "key" } else { "value" };
        return Err(Error::MissingField {
            line: line_no,
            token: keyword,
            field: field.to_string(),
        });
    };
    Ok((rest[..idx].to_string(), rest[idx..].trim().to_string()))
}

/// Liefert den Zeilen-Rest nach den ersten `n` Tokens, vorne und hinten
/// getrimmt; innere Leerzeichen bleiben erhalten.
///
/// Liefert `None`, wenn die Zeile weniger als `n` nicht-leere Tokens hat.
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

/// Parst `<style> <Koordinaten…> [width]` für die grafischen Tokens
/// (`LINE`, `RECTANGLE`, `CIRCLE`, `ARC`) in `.asy`-Dateien.
///
/// Stil-Tokens: `Normal` und `Wide` (→ `LineStyle::Bold`); `Dotted`/`Dashed`
/// werden in `.asy` abgelehnt ([Error::InvalidToken]). Das optionale
/// numerische Schluss-Argument ist die `width`; ein zweites zusätzliches
/// Token ist [Error::UnexpectedToken].
fn graphic_args(
    line_no: usize,
    keyword: &'static str,
    args: &[&str],
    n_coords: usize,
) -> Result<(crate::model::LineStyle, [i32; 8], Option<i32>), Error> {
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
    let style = match args[0] {
        s if s.eq_ignore_ascii_case("Normal") => crate::model::LineStyle::Normal,
        s if s.eq_ignore_ascii_case("Wide") => crate::model::LineStyle::Bold,
        _ => {
            return Err(Error::InvalidToken {
                line: line_no,
                token: keyword,
                field: String::from("style"),
                found: args[0].to_string(),
            })
        }
    };
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
    let width = args
        .get(base)
        .map(|tok| int32(line_no, "width", tok))
        .transpose()?;
    Ok((style, coords, width))
}

/// Parst ein Koordinaten-/Größen-Argument als `i32`.
///
/// Parse- und Range-Fehler werden zu [Error::InvalidNumber] umgewandelt.
fn int32(line: usize, field: &str, tok: &str) -> Result<i32, Error> {
    tok.parse::<i32>().map_err(|_| Error::InvalidNumber {
        line,
        field: field.to_string(),
        found: tok.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LineStyle;

    /// Erzeugt die `(Zeilennummer, Zeile)`-Paare eines Eingabe-Strings.
    fn lines_of<'a>(input: &'a str) -> impl Iterator<Item = (usize, String)> + 'a {
        input
            .split('\n')
            .enumerate()
            .map(|(i, l)| (i + 1, l.to_string()))
    }

    /// Parst einen String direkt zu einem [AsySymbol] (Testkürzel).
    fn parse(input: &str) -> Result<AsySymbol, Error> {
        parse_asy_lines(lines_of(input))
    }

    /// Die erste nicht-leere Zeile muss `Version` sein (leere Datei: Zeile 1).
    #[test]
    fn missing_version_is_an_error() {
        let err = parse("").unwrap_err();
        assert!(matches!(err, Error::MissingVersion { line: 1 }));
        let err = parse("\n\n").unwrap_err();
        assert!(matches!(err, Error::MissingVersion { line: 1 }));
        let err = parse("SYMATTR Value 1k").unwrap_err();
        assert!(matches!(err, Error::MissingVersion { line: 1 }));
        // Erste nicht-leere Zeile auf Zeile 3 → Fehler auf Zeile 3.
        let err = parse("\n\nSYMATTR Value 1k").unwrap_err();
        assert!(matches!(err, Error::MissingVersion { line: 3 }));
    }

    /// `Version` mit abweichender Argumentanzahl → Fehlereignisse.
    #[test]
    fn version_arg_count_is_enforced() {
        let err = parse("VERSION").unwrap_err();
        assert!(matches!(err, Error::MissingField { .. }));
        let err = parse("VERSION 4 extra").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 1, .. }));
    }

    /// `SYMBOLTYPE`: nur `CELL`/`BLOCK` (case-insensitiv, verbatim).
    #[test]
    fn symboltype_is_cell_or_block() {
        assert_eq!(
            parse("Version 4\nSYMBOLTYPE cell")
                .unwrap()
                .symbol_type
                .as_deref(),
            Some("cell")
        );
        assert_eq!(
            parse("Version 4\nSYMBOLTYPE BLOCK")
                .unwrap()
                .symbol_type
                .as_deref(),
            Some("BLOCK")
        );
        assert_eq!(parse("Version 4").unwrap().symbol_type.as_deref(), None);
        let err = parse("Version 4\nSYMBOLTYPE GURU").unwrap_err();
        assert!(matches!(err, Error::InvalidToken { line: 2, .. }));
        let err = parse("Version 4\nSYMBOLTYPE").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
        let err = parse("Version 4\nSYMBOLTYPE CELL extra").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 2, .. }));
    }

    /// `SYMATTR` speichert Key→Wert verbatim; letzter Wert gewinnt.
    #[test]
    fn symattr_properties() {
        let s =
            parse("Version 4\nSYMATTR VALUE 1k\nSYMATTR VALUE 2k\nSYMATTR SpiceLine  temp = 25\n")
                .unwrap();
        assert_eq!(s.properties.len(), 2);
        assert_eq!(s.properties.get("VALUE").map(String::as_str), Some("2k"));
        assert_eq!(
            s.properties.get("SpiceLine").map(String::as_str),
            Some("temp = 25")
        );
        let err = parse("Version 4\nSYMATTR Value").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
    }

    /// `WINDOW <index> <x> <y> <just> <n>`: exakt 5 Argumente wie in `.asc`;
    /// `(just)` kommt als `name`, `(n)` verbatim nach `visible`.
    #[test]
    fn window_parses_five_tokens() {
        let s = parse("Version 4\nWINDOW 0 31 40 Left 2\nWINDOW 39 -64 64 Center 1\n").unwrap();
        assert_eq!(
            s.windows,
            vec![
                WindowSpec {
                    index: 0,
                    x: 31,
                    y: 40,
                    name: "Left".to_string(),
                    visible: 2
                },
                WindowSpec {
                    index: 39,
                    x: -64,
                    y: 64,
                    name: "Center".to_string(),
                    visible: 1
                }
            ]
        );
        let err = parse("Version 4\nWINDOW 1 60 70 Left").unwrap_err();
        assert!(matches!(
            err,
            Error::MissingField {
                line: 2,
                token: "WINDOW",
                field,
                ..
            } if field == "visible"
        ));
        let err = parse("Version 4\nWINDOW 1 60 70").unwrap_err();
        assert!(matches!(
            err,
            Error::MissingField {
                line: 2,
                field,
                ..
            } if field == "justification"
        ));
        let err = parse("Version 4\nWINDOW 1 60 70 Left 2 3").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 2, .. }));
        let err = parse("Version 4\nWINDOW 1 x 70 Left 2").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { line: 2, .. }));
    }

    /// Grafische Primitive: `Normal` + `Wide` (→ `Bold`); `Dotted`/`Dashed`
    /// werden abgelehnt.
    #[test]
    fn graphics_styles() {
        for (tok, style) in [("Normal", LineStyle::Normal), ("Wide", LineStyle::Bold)] {
            let s = parse(&format!("Version 4\nLINE {tok} 0 0 10 20")).unwrap();
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
        for tok in ["Dotted", "Dashed"] {
            let err = parse(&format!("Version 4\nLINE {tok} 0 0 10 20")).unwrap_err();
            assert!(matches!(err, Error::InvalidToken { line: 2, .. }));
        }
    }

    /// Optionales `width`-Schlussargument wird als `flag` abgelegt.
    #[test]
    fn graphics_width_flag_is_kept() {
        let s = parse("Version 4\nLINE Normal 1 2 3 4 2\nARC Wide 1 2 3 4 5 6 7 8 1\n").unwrap();
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
                GraphicPrimitive::Arc {
                    style: LineStyle::Bold,
                    x1: 1,
                    y1: 2,
                    x2: 3,
                    y2: 4,
                    x3: 5,
                    y3: 6,
                    x4: 7,
                    y4: 8,
                    flag: Some(1)
                }
            ]
        );
        let err = parse("Version 4\nLINE Normal 1 2").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
        let err = parse("Version 4\nLINE Normal 1 2 3 4 2 3").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 2, .. }));
        let err = parse("Version 4\nLINE Normal 1 2 a b").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { line: 2, .. }));
    }

    /// `TEXT` voll: just + n (werde) + Text verbatim (innere Leerzeichen).
    #[test]
    fn text_full_form() {
        let s = parse("Version 4\nTEXT 16 40 Left 4 hello   world !dir").unwrap();
        assert_eq!(
            s.texts,
            vec![AsyText {
                x: 16,
                y: 40,
                justification: "Left".to_string(),
                text: "hello   world !dir".to_string()
            }]
        );
        // Tolerierte Kurzform `<x> <y> <text…>` (just = leer).
        let s = parse("Version 4\nTEXT 5 20 a label").unwrap();
        assert_eq!(
            s.texts,
            vec![AsyText {
                x: 5,
                y: 20,
                justification: String::new(),
                text: "a label".to_string()
            }]
        );
        // Drei-Argumente-Kurzform.
        let s = parse("Version 4\nTEXT 30 40 hello world").unwrap();
        assert_eq!(s.texts[0].text, "hello world");
        let err = parse("Version 4\nTEXT 1 2").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
        let err = parse("Version 4\nTEXT 1 2 Left 4").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
        let err = parse("Version 4\nTEXT 1 2 Left abc hello").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { line: 2, .. }));
    }

    /// `PIN` mit und ohne Name; `+`/`-`/`NC+` verbatim; genau 4/5 Argumente.
    #[test]
    fn pin_with_and_without_name() {
        let s = parse("Version 4\nPIN -5 20 NONE 0 +\nPIN 5 20 NC+ 4 -\nPIN 0 0 NONE 0\n").unwrap();
        assert_eq!(
            s.pins,
            vec![
                AsyPin {
                    x: -5,
                    y: 20,
                    justification: "NONE".to_string(),
                    offset: 0,
                    name: Some("+".to_string()),
                    properties: HashMap::new()
                },
                AsyPin {
                    x: 5,
                    y: 20,
                    justification: "NC+".to_string(),
                    offset: 4,
                    name: Some("-".to_string()),
                    properties: HashMap::new()
                },
                AsyPin {
                    x: 0,
                    y: 0,
                    justification: "NONE".to_string(),
                    offset: 0,
                    name: None,
                    properties: HashMap::new()
                }
            ]
        );
        let err = parse("Version 4\nPIN 1 2 NONE").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 2, .. }));
        let err = parse("Version 4\nPIN 1 2 NONE 0 a b").unwrap_err();
        assert!(matches!(err, Error::UnexpectedToken { line: 2, .. }));
        let err = parse("Version 4\nPIN a 2 NONE 0").unwrap_err();
        assert!(matches!(err, Error::InvalidNumber { line: 2, .. }));
    }

    /// `PINATTR` bindet an den letzten Pin; Duplikate: letzter gewinnt;
    /// ohne Pin → [Error::OrphanedPinAttr] mit Zeilennummer.
    #[test]
    fn pinattr_binds_to_last_pin() {
        let s = parse(
            "Version 4\n\
             PIN 1 2 NONE 0 A\n\
             PINATTR SPICEORDER 1\n\
             PINATTR SPICEORDER 2\n\
             PIN 3 4 RIGHT 0 B\n\
             PINATTR NAME my pin\n",
        )
        .unwrap();
        assert_eq!(s.pins.len(), 2);
        assert_eq!(
            s.pins[0].properties.get("SPICEORDER").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            s.pins[1].properties.get("NAME").map(String::as_str),
            Some("my pin")
        );
        let err = parse("Version 4\nPINATTR SPICEORDER 1").unwrap_err();
        assert!(matches!(err, Error::OrphanedPinAttr { line: 2 }));
        let err = parse("Version 4\nPIN 1 2 NONE 0\nPINATTR NAME").unwrap_err();
        assert!(matches!(err, Error::MissingField { line: 3, .. }));
    }

    /// Unbekannte Zeilen (DRAW, ATTRDEF, SYMKEY, IOPIN, …) → ignoriert.
    #[test]
    fn unknown_lines_are_ignored() {
        let s = parse(
            "Version 4\n\
             SYMATTR NAME res\n\
             DRAW 0 0 -10 22 0 22 2\n\
             ATTRDEF 0\n\
             ATTRNAME VALUE\n\
             SYMKEY 0 10 0\n\
             PIN 10 22 NONE 0\n",
        )
        .unwrap();
        assert!(s.properties.contains_key("NAME"));
        assert!(s.texts.is_empty());
        assert!(s.pins.len() == 1);
    }

    /// Schlüsselwörter sind case-insensitiv.
    #[test]
    fn keywords_are_case_insensitive() {
        let s = parse("VERSION 4\nsymboltype CELL\nline normal 0 0 1 1\npin 5 2 none 0").unwrap();
        assert_eq!(s.version, "4");
        assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
        assert_eq!(s.graphics.len(), 1);
        assert_eq!(s.pins.len(), 1);
        assert_eq!(s.pins[0].justification, "none");
    }

    /// Vollständige synthetische Datei: alle Felder zusammen.
    #[test]
    fn full_synthetic_file() {
        let s = parse(
            "Version 4\n\
             SYMBOLTYPE CELL\n\
             SYMATTR NAME res\n\
             WINDOW 1 32 38 Left 2\n\
             LINE Normal 0 22 10 22\n\
             RECTANGLE Wide 10 10 40 34 2\n\
             CIRCLE Normal 50 0 60 10\n\
             ARC Wide 1 2 3 4 5 6 7 8\n\
             TEXT 16 40 Left 4 1kΩ\n\
             PIN 0 22 NONE 0 +\n\
             PINATTR SPICEORDER 1\n\
             PIN 50 22 NONE 0 -\n",
        )
        .unwrap();
        assert_eq!(s.version, "4");
        assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
        assert_eq!(s.properties.get("NAME").map(String::as_str), Some("res"));
        assert_eq!(s.windows.len(), 1);
        assert_eq!(s.graphics.len(), 4);
        assert_eq!(s.texts.len(), 1);
        assert_eq!(s.texts[0].text, "1kΩ");
        assert_eq!(s.pins.len(), 2);
        assert!(s.pins[0].name.is_some());
        // `PINATTR` bindet an den letzten `PIN` (hier: Pin 0).
        assert!(s.pins[0].properties.contains_key("SPICEORDER"));
        assert!(!s.pins[1].properties.contains_key("SPICEORDER"));
    }
}
