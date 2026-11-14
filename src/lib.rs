//! # webspice
//!
//! Parser-Library für LTspice `.asc`-Schematic-Dateien: liest eine Datei,
//! einen Byte-Buffer oder einen Strom ein und liefert das strukturierte
//! [Schematic]-AST. Zusätzlich: LTspice-Symboldefinitionen (`.asy`) als
//! [AsySymbol] (Abschnitt „Symbol-Rendering (.asy)" der Dokumentation).
//!
//! ```
//! use webspice::parse_bytes;
//!
//! let result = parse_bytes(b"VERSION 4\nWIRE 160 96 160 160\n");
//! assert!(result.is_ok());
//! ```
//!
//! Details: `docs/documentation.md` (Single Source of Truth).

#![deny(missing_docs)]

mod asy;
mod error;
mod model;
mod parser;

#[cfg(target_arch = "wasm32")]
mod wasm;

use std::io::BufRead;

pub use crate::asy::{AsyPin, AsySymbol, AsyText};
pub use crate::error::Error;
pub use crate::model::{
    Bustap, Flag, GraphicPrimitive, LineStyle, Orientation, Schematic, Sheet, Symbol,
    TextDirective, WindowSpec, Wire,
};

/// Liest eine `.asc`-Datei ein und parst sie zu einem [Schematic].
pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Schematic, Error> {
    let data = std::fs::read(path.as_ref())?;
    parse_bytes(&data)
}

/// Parst `.asc`-Inhalt aus einem `&[u8]`-Buffer.
///
/// Nicht-UTF-8-Bytes werden verlustfrei als Windows-1252 (cp1252) dekodiert
/// (z. B. `0xB5` = µ in `SYMATTR Value 10µ`); siehe Dokumentation, Abschnitt
/// „API".
pub fn parse_bytes(data: &[u8]) -> Result<Schematic, Error> {
    parse_str(&decode_windows1252_fallback(data))
}

/// Parst `.asc`-Inhalt aus einem `BufRead`-Strom.
///
/// Encoding wie bei [crate::parse_bytes].
pub fn parse_reader<R: BufRead>(mut reader: R) -> Result<Schematic, Error> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    parse_str(&decode_windows1252_fallback(&buf))
}

/// Parst eine LTspice-Symboldefinition (`.asy`) aus einem `&[u8]`-Buffer
/// zu einem [AsySymbol].
///
/// Encoding wie bei [crate::parse_bytes] (cp1252-Fallback); Parsing-Regeln
/// siehe `docs/documentation.md`, Abschnitt „Symbol-Rendering (.asy)".
pub fn parse_asy_bytes(data: &[u8]) -> Result<AsySymbol, Error> {
    parse_asy_str(&decode_windows1252_fallback(data))
}

/// Gemeinsame Basis für `.asy`: BOM ignorieren, Zeilen aufteilen und an
/// [crate::asy::parse_asy_lines] übergeben (Zeilennummern 1-basiert).
fn parse_asy_str(text: &str) -> Result<AsySymbol, Error> {
    let text = text.trim_start_matches('\u{feff}');
    let lines = text
        .split('\n')
        .enumerate()
        .map(|(idx, line)| (idx + 1, line.to_string()));
    asy::parse_asy_lines(lines)
}

/// cp1252-Mapping für die Bytes 0x80–0x9F (Rest der Tabelle identisch mit
/// Latin-1; die in cp1252 undefinierten Einträge mappen verlustfrei auf
/// C1-Kontrollzeichen, wie im WHATWG-Encoding-Standard).
const CP1252_080_TO_09F: [char; 32] = [
    '\u{20AC}', // 0x80 €
    '\u{81}',   // 0x81 (undefiniert)
    '\u{201A}', // 0x82 ‚
    '\u{0192}', // 0x83 ƒ
    '\u{201E}', // 0x84 „
    '\u{2026}', // 0x85 …
    '\u{2020}', // 0x86 †
    '\u{2021}', // 0x87 ‡
    '\u{02C6}', // 0x88 ˆ
    '\u{2030}', // 0x89 ‰
    '\u{0160}', // 0x8A Š
    '\u{2039}', // 0x8B ‹
    '\u{0152}', // 0x8C Œ
    '\u{8D}',   // 0x8D (undefiniert)
    '\u{017D}', // 0x8E Ž
    '\u{8F}',   // 0x8F (undefiniert)
    '\u{90}',   // 0x90 (undefiniert)
    '\u{2018}', // 0x91 ‘
    '\u{2019}', // 0x92 ’
    '\u{201C}', // 0x93 „ (links, doppel)
    '\u{201D}', // 0x94 " (rechts, doppel)
    '\u{2022}', // 0x95 •
    '\u{2013}', // 0x96 –
    '\u{2014}', // 0x97 —
    '\u{02DC}', // 0x98 ˜
    '\u{2122}', // 0x99 ™
    '\u{0161}', // 0x9A š
    '\u{203A}', // 0x9B ›
    '\u{0153}', // 0x9C œ
    '\u{9D}',   // 0x9D (undefiniert)
    '\u{017E}', // 0x9E ž
    '\u{0178}', // 0x9F Ÿ
];

/// Dekodiert `.asc`-Bytes nach Unicode.
///
/// Gültiges UTF-8 bleibt unverändert; andernfalls fällt die Gesamteingabe
/// auf Windows-1252 (cp1252) zurück: Bytes 0x00–0x7F bleiben gleich,
/// 0x80–0x9F folgen [CP1252_080_TO_09F], 0xA0–0xFF sind identisch mit
/// Latin-1. Alle 256 Byte-Werte sind gültige `char`-Abbildungen, also ist
/// der Fallback verlustfrei.
fn decode_windows1252_fallback(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    bytes
        .iter()
        .map(|&b| {
            if (0x80..0xA0).contains(&b) {
                CP1252_080_TO_09F[(b - 0x80) as usize]
            } else {
                b as char
            }
        })
        .collect()
}

/// Serialisiert ein [Schematic] als JSON-String.
///
/// Der JSON-Vertrag (Tastennamen, `null` für fehlende Felder,
/// Token-Strings für Enums) steht im Abschnitt "WASM / Web-Export"
/// von `docs/documentation.md`; konsumiert wird er vom Web-Frontend.
pub fn schematic_to_json(s: &Schematic) -> Result<String, Error> {
    Ok(serde_json::to_string(s)?)
}

/// Serialisiert einen [AsySymbol] als JSON-String (Web-Export).
///
/// Der JSON-Vertrag (Tastennamen, fehlende Pin-Namen) steht im Abschnitt
/// „JSON-Format `AsySymbol` (WASM-Vertrag)" von `docs/documentation.md`.
pub fn asy_to_json(symbol: &AsySymbol) -> Result<String, Error> {
    Ok(serde_json::to_string(symbol)?)
}

/// Gemeinsame Basis: BOM ignorieren, Zeilen aufteilen und an
/// [crate::parser::parse_lines] übergeben (Zeilennummern 1-basiert).
fn parse_str(text: &str) -> Result<Schematic, Error> {
    let text = text.trim_start_matches('\u{feff}');
    let lines = text
        .split('\n')
        .enumerate()
        .map(|(idx, line)| (idx + 1, line.to_string()));
    parser::parse_lines(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leerer Eingabepuffer → leeres Schematic.
    #[test]
    fn empty_input_yields_empty_schematic() {
        let s = parse_bytes(b"").unwrap();
        assert_eq!(s, Schematic::default());
    }

    /// Nur-Whitespace-Inhalt → leeres Schematic.
    #[test]
    fn whitespace_only_input_yields_empty_schematic() {
        let s = parse_bytes(b"   \n\t\n  \r\n").unwrap();
        assert_eq!(s, Schematic::default());
    }

    /// Unbekannte Tokens bleiben roh in `Schematic.extras` erhalten.
    #[test]
    fn unknown_tokens_are_kept_raw() {
        let s = parse_bytes(b"FOOBAR 1 2 3\nNEWTON 99\n").unwrap();
        assert_eq!(
            s,
            Schematic {
                extras: vec!["FOOBAR 1 2 3".to_string(), "NEWTON 99".to_string()],
                ..Default::default()
            }
        );
    }

    /// CRLF und LF erzeugen dasselbe Ergebnis.
    #[test]
    fn crlf_and_lf_are_equivalent() {
        let lf = parse_bytes(b"VERSION 4\nSHEET 1 880 680\n").unwrap();
        let crlf = parse_bytes(b"VERSION 4\r\nSHEET 1 880 680\r\n").unwrap();
        assert_eq!(lf, crlf);
    }

    /// Tab-getrennte Argumente werden wie Leerzeichen behandelt.
    #[test]
    fn tab_separated_args_accepted() {
        let s = parse_bytes(b"SHEET\t1\t880\t680").unwrap();
        assert_eq!(
            s.sheet,
            Some(Sheet {
                page: 1,
                width: 880,
                height: 680
            })
        );
    }

    /// cp1252-`0xB5` = µ — realer LTspice-Fall (`SYMATTR Value 10µ`).
    #[test]
    fn cp1252_micro_sign_in_value_decodes() {
        let s = parse_bytes(b"SYMBOL res 200 100 R0\nSYMATTR Value 10\xB5\n").unwrap();
        assert_eq!(
            s.symbols[0].attributes.get("Value").map(String::as_str),
            Some("10\u{B5}")
        );
    }

    /// cp1252-spezifische Zeichen aus 0x80–0x9F (€, •, „, ", —).
    #[test]
    fn cp1252_specific_characters_decode() {
        let s = parse_bytes(b"TEXT 0 0 Left 1 A\x80B\x95D\x93E\x94F\x97\n").unwrap();
        assert_eq!(
            s.texts[0].text.as_str(),
            "A\u{20AC}B\u{2022}D\u{201C}E\u{201D}F\u{2014}"
        );
    }

    /// Undefinierte cp1252-Byte (0x81) und hohe Bytes (0xFF) dekodieren
    /// verlustfrei ohne Fehler (C1-Kontrollzeichen / U+00FF).
    #[test]
    fn high_bytes_decode_losslessly_without_error() {
        let s = parse_bytes(&[0x81, 0xFF, 0xFE]).unwrap();
        assert_eq!(
            s,
            Schematic {
                extras: vec!["\u{81}\u{FF}\u{FE}".to_string()],
                ..Default::default()
            }
        );
    }

    /// Nicht vorhandene Datei → `Io`-Fehler.
    #[test]
    fn missing_file_is_io_error() {
        let err = parse_file("definitely/not/here.asc").unwrap_err();
        assert!(matches!(err, Error::Io(_)));
    }

    /// UTF-8-BOM am Datei-Anfang wird ignoriert.
    #[test]
    fn bom_at_start_is_ignored() {
        let mut with_bom = vec![0xEFu8, 0xBB, 0xBF];
        with_bom.extend_from_slice(b"VERSION 4\nWIRE 1 2 3 4\n");
        let without = b"VERSION 4\nWIRE 1 2 3 4\n".to_vec();
        assert_eq!(
            parse_bytes(&with_bom).unwrap(),
            parse_bytes(&without).unwrap()
        );
    }

    /// Datei mit nur BOM → leeres Schematic.
    #[test]
    fn bom_only_yields_empty_schematic() {
        let s = parse_bytes(&[0xEF, 0xBB, 0xBF]).unwrap();
        assert_eq!(s, Schematic::default());
    }

    /// Sehr lange Zeile (~100k Zeichen) parst ohne Hängen.
    #[test]
    fn very_long_line_parses() {
        let content = "a".repeat(100_000);
        let line = format!("TEXT 0 0 Left 1 {content}");
        let s = parse_bytes(line.as_bytes()).unwrap();
        assert_eq!(s.texts.len(), 1);
        assert_eq!(s.texts[0].text.len(), 100_000);
    }

    /// `parse_reader` und `parse_bytes` produzieren dasselbe Ergebnis.
    #[test]
    fn parse_reader_matches_parse_bytes() {
        let input = "VERSION 4\nWIRE 1 2 3 4\n";
        let from_reader = parse_reader(std::io::Cursor::new(input.as_bytes())).unwrap();
        let from_bytes = parse_bytes(input.as_bytes()).unwrap();
        assert_eq!(from_reader, from_bytes);
    }

    /// Leeres Schematic → exakter, stabiler JSON-String.
    #[test]
    fn empty_schematic_serializes_to_expected_json() {
        let s = Schematic::default();
        let json = schematic_to_json(&s).unwrap();
        assert_eq!(
            json,
            r#"{"version":null,"sheet":null,"wires":[],"flags":[],"bustaps":[],"symbols":[],"texts":[],"graphics":[],"extras":[]}"#
        );
    }

    /// Mix-Schematic → JSON-Struktur entspricht exakt dem AST
    /// (Feld-feld, inkl. Enum-Token-Strings und Attribute-Objekt).
    #[test]
    fn mixed_schematic_serializes_to_expected_json_shape() {
        let input = [
            "VERSION 4.1",
            "SHEET 1 880 680",
            "WIRE 160 96 224 96",
            "FLAG 160 96 net1",
            "BUSTAP 300 200",
            "SYMBOL res 200 100 R90",
            "SYMATTR InstName R1",
            "SYMATTR Value 1k",
            "TEXT 10 20 Left 1 !.tran 10m",
            "LINE Normal 0 0 10 10 2",
            "RECTANGLE Dotted 100 100 200 150",
            "CIRCLE Bold 50 50 80 70",
            "ARC Dashed 0 0 100 50 0 25 100 25",
        ]
        .join("\n");
        let s = parse_bytes(input.as_bytes()).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&schematic_to_json(&s).unwrap()).unwrap();

        let expected = serde_json::json!({
            "version": "4.1",
            "sheet": { "page": 1, "width": 880, "height": 680 },
            "wires": [{ "x1": 160, "y1": 96, "x2": 224, "y2": 96 }],
            "flags": [{ "x": 160, "y": 96, "name": "net1" }],
            "bustaps": [{ "x": 300, "y": 200 }],
            "symbols": [{
                "name": "res",
                "x": 200,
                "y": 100,
                "orientation": "R90",
                "attributes": { "InstName": "R1", "Value": "1k" },
                "windows": [],
                "extras": []
            }],
            "texts": [{
                "x": 10, "y": 20, "align": "Left", "size": 1,
                "is_command": true, "text": "!.tran 10m"
            }],
            "graphics": [
                { "Line": { "style": "Normal", "x1": 0, "y1": 0, "x2": 10, "y2": 10, "flag": 2 } },
                { "Rectangle": { "style": "Dotted", "x1": 100, "y1": 100, "x2": 200, "y2": 150, "flag": null } },
                { "Circle": { "style": "Bold", "x1": 50, "y1": 50, "x2": 80, "y2": 70, "flag": null } },
                { "Arc": { "style": "Dashed", "x1": 0, "y1": 0, "x2": 100, "y2": 50,
                          "x3": 0, "y3": 25, "x4": 100, "y4": 25, "flag": null } }
            ],
            "extras": []
        });
        assert_eq!(value, expected);
    }

    /// Fehlende `version`/`sheet` landen als JSON-`null` (nicht abwesend).
    #[test]
    fn missing_optional_fields_serialize_as_null() {
        let s = parse_bytes(b"WIRE 1 2 3 4\n").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&schematic_to_json(&s).unwrap()).unwrap();
        assert!(value["version"].is_null());
        assert!(value["sheet"].is_null());
    }
}
