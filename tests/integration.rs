//! Integrationstests: synthetische `.asc`-Fixtures (in `tests/fixtures/`)
//! werden via `parse_file` gelesen und strukturell geprüft; die
//! Fehler-Fixtures liefern die exakte Fehler-Variante mit Zeilennummer.
//!
//! Siehe `docs/documentation.md` (Abschnitt "Required test cases",
//! Block "Ab Task 06").

use std::path::{Path, PathBuf};

use webspice::{
    AsySymbol, Error, Flag, GraphicPrimitive, LineStyle, Orientation, Schematic, Sheet, Wire,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// `empty.asc` → leeres Schematic.
#[test]
fn empty_fixture_parses_to_default() {
    let s = webspice::parse_file(fixture("empty.asc")).unwrap();
    assert_eq!(s, Schematic::default());
}

/// `meta-only.asc` → nur Version und Seite sind gesetzt.
#[test]
fn meta_only_fixture_sets_version_and_sheet() {
    let s = webspice::parse_file(fixture("meta-only.asc")).unwrap();
    assert_eq!(s.version.as_deref(), Some("4"));
    assert_eq!(
        s.sheet,
        Some(Sheet {
            page: 1,
            width: 880,
            height: 680
        })
    );
    assert_eq!(s.wires.len(), 0);
    assert_eq!(s.flags.len(), 0);
    assert_eq!(s.bustaps.len(), 0);
    assert_eq!(s.symbols.len(), 0);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 0);
}

/// `resistor-bridge.asc` → Mix aus Wires, Flags, Symbol und Text.
#[test]
fn resistor_bridge_fixture_structure() {
    let s = webspice::parse_file(fixture("resistor-bridge.asc")).unwrap();
    assert_eq!(
        s.wires,
        vec![
            Wire {
                x1: 160,
                y1: 96,
                x2: 160,
                y2: 160
            },
            Wire {
                x1: 160,
                y1: 160,
                x2: 224,
                y2: 160
            }
        ]
    );
    assert_eq!(
        s.flags,
        vec![
            Flag {
                x: 160,
                y: 96,
                name: "0".to_string()
            },
            Flag {
                x: 224,
                y: 160,
                name: "V1".to_string()
            }
        ]
    );
    assert_eq!(s.symbols.len(), 1);
    let sym = &s.symbols[0];
    assert_eq!(sym.name, "res");
    assert_eq!(sym.orientation, Orientation::R0);
    assert_eq!(
        sym.attributes.get("InstName").map(String::as_str),
        Some("R1")
    );
    assert_eq!(sym.attributes.get("Value").map(String::as_str), Some("1k"));
    assert_eq!(
        sym.attributes.get("SpiceLine").map(String::as_str),
        Some("temp=25")
    );
    assert_eq!(s.texts.len(), 1);
    assert!(s.texts[0].is_command);
    assert_eq!(s.texts[0].text, "!.tran 10m");
    assert_eq!(s.graphics.len(), 0);
}

/// `all-graphics.asc` → alle vier Primitives, jedes einen anderen Style.
#[test]
fn all_graphics_fixture_has_every_primitive_and_style() {
    let s = webspice::parse_file(fixture("all-graphics.asc")).unwrap();
    assert_eq!(
        s.graphics,
        vec![
            GraphicPrimitive::Line {
                style: LineStyle::Normal,
                x1: 0,
                y1: 0,
                x2: 10,
                y2: 20,
                flag: None
            },
            GraphicPrimitive::Rectangle {
                style: LineStyle::Dotted,
                x1: 0,
                y1: 0,
                x2: 10,
                y2: 20,
                flag: None
            },
            GraphicPrimitive::Circle {
                style: LineStyle::Dashed,
                x1: 0,
                y1: 0,
                x2: 10,
                y2: 20,
                flag: None
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
                flag: None
            }
        ]
    );
}

/// `weird.asc` → CRLF, Tabs, unbekannte Tokens und Umlaute: kein Fehler,
/// Inhalte bleiben erhalten.
#[test]
fn weird_fixture_is_tolerated() {
    let s = webspice::parse_file(fixture("weird.asc")).unwrap();
    assert_eq!(s.version.as_deref(), Some("4"));
    assert_eq!(
        s.sheet,
        Some(Sheet {
            page: 1,
            width: 880,
            height: 680
        })
    );
    assert_eq!(
        s.flags,
        vec![
            Flag {
                x: 10,
                y: 10,
                name: "Masse".to_string()
            },
            Flag {
                x: 20,
                y: 20,
                name: "GND".to_string()
            }
        ]
    );
    assert_eq!(
        s.symbols[0].attributes.get("InstName").map(String::as_str),
        Some("R_ü")
    );
    assert_eq!(s.texts.len(), 1);
    assert!(!s.texts[0].is_command);
    assert_eq!(s.texts[0].text, "Begrüßung schön");
    // Unbekannte Tokens (MYSTERY/NEWTON) bleiben roh in
    // `schematic.extras` erhalten:
    assert_eq!(s.extras, vec!["MYSTERY 42", "NEWTON 99"]);
    assert_eq!(s.bustaps.len(), 0);
    assert_eq!(s.wires.len(), 0);
}

/// `errors/orphan-symattr.asc` → `OrphanedSymAttr` mit Zeilennummer 1.
#[test]
fn orphan_symattr_fixture_reports_line() {
    let err = webspice::parse_file(fixture("errors/orphan-symattr.asc")).unwrap_err();
    assert!(
        matches!(err, Error::OrphanedSymAttr { line: 1 }),
        "unexpected: {err:?}"
    );
}

/// `errors/bad-number.asc` → `InvalidNumber` mit Zeilennummer 3 und Wert.
#[test]
fn bad_number_fixture_reports_line_and_value() {
    let err = webspice::parse_file(fixture("errors/bad-number.asc")).unwrap_err();
    match &err {
        Error::InvalidNumber {
            line: 3,
            field,
            found,
        } => {
            assert_eq!(field, "x2");
            assert_eq!(found, "abc");
        }
        other => panic!("expected InvalidNumber on line 3, got: {other:?}"),
    }
}

/// `errors/bad-orientation.asc` → `InvalidToken` mit Zeilennummer 1.
#[test]
fn bad_orientation_fixture_reports_line_and_token() {
    let err = webspice::parse_file(fixture("errors/bad-orientation.asc")).unwrap_err();
    match &err {
        Error::InvalidToken {
            line: 1,
            field,
            found,
            ..
        } => {
            assert_eq!(field, "orientation");
            assert_eq!(found, "R45");
        }
        other => panic!("expected InvalidToken on line 1, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// `.asy`-Symbol-Definitionen (Task 09)
// ---------------------------------------------------------------------------

fn asy_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/asy-fixtures")
        .join(name)
}

/// Liest eine `.asy`-Fixture per `parse_asy_bytes` ein und parst sie.
fn parse_asy_fixture(name: &str) -> Result<AsySymbol, Error> {
    let bytes = std::fs::read(asy_fixture(name)).unwrap();
    webspice::parse_asy_bytes(&bytes)
}

fn line(style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32) -> GraphicPrimitive {
    GraphicPrimitive::Line {
        style,
        x1,
        y1,
        x2,
        y2,
        flag: None,
    }
}

fn rect(style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32) -> GraphicPrimitive {
    GraphicPrimitive::Rectangle {
        style,
        x1,
        y1,
        x2,
        y2,
        flag: None,
    }
}

/// `res.asy` → 6 Linien, 2 Windows, 2 Pins mit `PINATTR`-Bindung.
#[test]
fn asy_res_fixture_structure() {
    let s = parse_asy_fixture("res.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.properties.get("Value").map(String::as_str), Some("R"));
    assert_eq!(s.properties.get("Prefix").map(String::as_str), Some("R"));
    assert_eq!(
        s.properties.get("Description").map(String::as_str),
        Some("A Resistor(European style graphic)")
    );
    assert_eq!(s.windows.len(), 2);
    assert_eq!(
        s.windows[0],
        webspice::WindowSpec {
            index: 0,
            x: 31,
            y: 40,
            name: "Left".to_string(),
            visible: 2
        }
    );
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 6);
    assert!(matches!(s.graphics[0], GraphicPrimitive::Line { .. }));
    assert_eq!(s.graphics[0], line(LineStyle::Normal, 16, 88, 16, 96));
    assert_eq!(s.pins.len(), 2);
    let p0 = &s.pins[0];
    assert_eq!((p0.x, p0.y), (16, 16));
    assert_eq!(p0.name, None);
    assert_eq!(p0.properties.get("PinName").map(String::as_str), Some("A"));
    assert_eq!(
        p0.properties.get("SpiceOrder").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        s.pins[1].properties.get("PinName").map(String::as_str),
        Some("B")
    );
}

/// `cap.asy` → 2 Linien + 2 Rechtecke, 2 Pins.
#[test]
fn asy_cap_fixture_structure() {
    let s = parse_asy_fixture("cap.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 2);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 4);
    assert_eq!(s.graphics[2], rect(LineStyle::Normal, 0, 36, 32, 44));
    assert_eq!(s.pins.len(), 2);
    assert_eq!(
        s.pins[0].properties.get("PinName").map(String::as_str),
        Some("A")
    );
    assert_eq!(
        s.pins[1].properties.get("PinName").map(String::as_str),
        Some("B")
    );
}

/// `diode.asy` → 6 Linien, Pins `+`/`-` werfen keine Fehler.
#[test]
fn asy_diode_fixture_pin_names_with_polarity() {
    let s = parse_asy_fixture("diode.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 2);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 6);
    assert!(s
        .graphics
        .iter()
        .all(|g| matches!(g, GraphicPrimitive::Line { .. })));
    assert_eq!(s.pins.len(), 2);
    assert_eq!(
        s.pins[0].properties.get("PinName").map(String::as_str),
        Some("+")
    );
    assert_eq!(
        s.pins[1].properties.get("PinName").map(String::as_str),
        Some("-")
    );
}

/// `npn.asy` → 7 Linien, 3 Pins `C`/`B`/`E` mit `SpiceOrder`.
#[test]
fn asy_npn_fixture_three_pinned_transistor() {
    let s = parse_asy_fixture("npn.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 2);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 7);
    assert!(s
        .graphics
        .iter()
        .all(|g| matches!(g, GraphicPrimitive::Line { .. })));
    assert_eq!(s.pins.len(), 3);
    let names: Vec<&str> = s
        .pins
        .iter()
        .map(|p| p.properties.get("PinName").unwrap().as_str())
        .collect();
    assert_eq!(names, vec!["C", "B", "E"]);
    assert_eq!(
        s.pins[0].properties.get("SpiceOrder").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        s.pins[2].properties.get("SpiceOrder").map(String::as_str),
        Some("3")
    );
}

/// `pot.asy` + `pot_log.asy` → 10 Linien, 3 `TEXT`-Zeilen (Vollform),
/// negative Koordinaten und `SpiceLine*`-Attribute.
#[test]
fn asy_pot_fixtures_with_texts_and_spicelines() {
    for name in ["pot.asy", "pot_log.asy"] {
        let s = parse_asy_fixture(name).unwrap();
        assert_eq!(s.version, "4");
        assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
        assert_eq!(s.windows.len(), 2);
        assert_eq!(
            s.properties.get("SpiceLine").map(String::as_str),
            Some("R=1Meg Val=50")
        );
        assert_eq!(s.graphics.len(), 10);
        assert!(s
            .graphics
            .iter()
            .all(|g| matches!(g, GraphicPrimitive::Line { .. })));
        // Mindestens eine Koordinate muss negativ sein (z. B. `-11`, `-32`),
        // damit die BBox/Rendering echte negative Werte übt:
        let any_negative = s.graphics.iter().any(|g| match g {
            GraphicPrimitive::Line { x1, y1, x2, y2, .. } => {
                *x1 < 0 || *y1 < 0 || *x2 < 0 || *y2 < 0
            }
            _ => false,
        });
        assert!(any_negative, "{name}: erwartete negative Koordinate");
        assert_eq!(s.texts.len(), 3);
        assert_eq!(s.pins.len(), 3);
    }
    // `pot.asy`: 3. Text = `linear` über die `lin`-Textzeile.
    let pot = parse_asy_fixture("pot.asy").unwrap();
    assert_eq!(pot.texts[2].text, "lin");
    let pot_log = parse_asy_fixture("pot_log.asy").unwrap();
    assert_eq!(pot_log.texts[2].text, "log");
}

/// `opamp.asy` → 1 `WINDOW` (Index 0), 3 Pins, `SpiceLine`/`SpiceLine2`.
#[test]
fn asy_opamp_fixture_spice_lines() {
    let s = parse_asy_fixture("opamp.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 1);
    assert_eq!(s.windows[0].name, "Left");
    assert_eq!(s.windows[0].visible, 2);
    assert_eq!(
        s.properties.get("SpiceLine").map(String::as_str),
        Some("Aol=100K")
    );
    assert_eq!(
        s.properties.get("SpiceLine2").map(String::as_str),
        Some("GBW=10Meg")
    );
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 6);
    // `opamp.asy` enthält negative X-Koordinaten (z. B. `-32`):
    let any_negative_x = s.graphics.iter().any(|g| match g {
        GraphicPrimitive::Line { x1, x2, .. } => *x1 < 0 || *x2 < 0,
        _ => false,
    });
    assert!(
        any_negative_x,
        "opamp asy soll negative X-Koordinaten enthalten"
    );
    assert_eq!(s.pins.len(), 3);
    let names: Vec<&str> = s
        .pins
        .iter()
        .map(|p| p.properties.get("PinName").unwrap().as_str())
        .collect();
    assert_eq!(names, vec!["invin", "noninvin", "out"]);
}

/// `EuropeanPolcap.asy` → 14 Primitive (13 Linien + 1 Rechteck), 2 Pins.
#[test]
fn asy_european_polcap_fixture_many_lines_plus_rect() {
    let s = parse_asy_fixture("EuropeanPolcap.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 2);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 14);
    let lines = s
        .graphics
        .iter()
        .filter(|g| matches!(g, GraphicPrimitive::Line { .. }))
        .count();
    let rects = s
        .graphics
        .iter()
        .filter(|g| matches!(g, GraphicPrimitive::Rectangle { .. }))
        .count();
    assert_eq!(lines, 13);
    assert_eq!(rects, 1);
    assert_eq!(s.pins.len(), 2);
}

/// `EuropeanResistor.asy` → identisch zu `res.asy` (6 Linien, 2 Pins).
#[test]
fn asy_european_resistor_fixture_matches_res_shape() {
    let s = parse_asy_fixture("EuropeanResistor.asy").unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.windows.len(), 2);
    assert_eq!(s.texts.len(), 0);
    assert_eq!(s.graphics.len(), 6);
    assert!(s
        .graphics
        .iter()
        .all(|g| matches!(g, GraphicPrimitive::Line { .. })));
    assert_eq!(s.pins.len(), 2);
}

/// `asy_to_json` → exakter JSON-Wert (WASM-Vertrag) für `res.asy`.
///
/// Tasten = exakte Rust-Feldnamen (snake_case), `symbolType` ist
/// umgezogen (keil), `GraphicPrimitive` ist extern getaggt (`"Line"`),
/// `LineStyle` = Token-String; Pin ohne `name`-Schlüssel.
#[test]
fn asy_json_contract_for_res() {
    let s = parse_asy_fixture("res.asy").unwrap();
    let json = webspice::asy_to_json(&s).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let expected = serde_json::json!({
        "version": "4",
        "symbolType": "CELL",
        "properties": {
            "Value": "R",
            "Prefix": "R",
            "Description": "A Resistor(European style graphic)"
        },
        "windows": [
            {"index": 0, "x": 31, "y": 40, "name": "Left", "visible": 2},
            {"index": 3, "x": 31, "y": 76, "name": "Left", "visible": 2}
        ],
        "graphics": [
            {"Line": {"style": "Normal", "x1": 16, "y1": 88, "x2": 16, "y2": 96, "flag": null}},
            {"Line": {"style": "Normal", "x1": 16, "y1": 16, "x2": 16, "y2": 24, "flag": null}},
            {"Line": {"style": "Normal", "x1": 27, "y1": 24, "x2": 5,  "y2": 24, "flag": null}},
            {"Line": {"style": "Normal", "x1": 5,  "y1": 88, "x2": 5,  "y2": 24, "flag": null}},
            {"Line": {"style": "Normal", "x1": 5,  "y1": 88, "x2": 27, "y2": 88, "flag": null}},
            {"Line": {"style": "Normal", "x1": 27, "y1": 24, "x2": 27, "y2": 88, "flag": null}}
        ],
        "texts": [],
        "pins": [
            {
                "x": 16, "y": 16, "justification": "NONE", "offset": 0,
                "properties": {"PinName": "A", "SpiceOrder": "1"}
            },
            {
                "x": 16, "y": 96, "justification": "NONE", "offset": 0,
                "properties": {"PinName": "B", "SpiceOrder": "2"}
            }
        ]
    });
    assert_eq!(value, expected, "JSON-Vertrag für res.asy abweicht\n{json}");
}

/// Empty file → `MissingVersion` mit exaktem Display.
#[test]
fn asy_empty_file_reports_missing_version_line_1() {
    let err = webspice::parse_asy_bytes(b"").unwrap_err();
    let msg = err.to_string();
    assert_eq!(msg, "line 1: missing required Version line");
}

/// `PINATTR` ohne vorhergehendes `PIN` → `OrphanedPinAttr` mit exakter Zeilennummer.
#[test]
fn asy_orphaned_pinattr_reports_line() {
    // `PINATTR` ist hier Zeile 5; kein `PIN` davor → orphan.
    let bytes =
        b"Version 4\nSymbolType CELL\nSYMATTR Value R\nSYMATTR Prefix R\nPINATTR PinName A\n";
    let err = webspice::parse_asy_bytes(bytes).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.starts_with("line 5: PINATTR without a preceding PIN"),
        "unexpected: {msg}"
    );
}

/// BOM am Datei-Anfang wird ignoriert (`.asy`-Pfad).
#[test]
fn asy_bom_is_tolerated() {
    let bytes = b"\xEF\xBB\xBFVersion 4\nSymbolType CELL\nSYMATTR Value R\nPIN 16 16 NONE 0\nPINATTR PinName A\n";
    let s = webspice::parse_asy_bytes(bytes).unwrap();
    assert_eq!(s.version, "4");
    assert_eq!(s.symbol_type.as_deref(), Some("CELL"));
    assert_eq!(s.pins.len(), 1);
}
