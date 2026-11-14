//! Browser-Export (wasm32-unknown-unknown only).
//!
//! Export-Funktionen: [`parse`] (`.asc`-Bytes → JSON des [Schematic]-AST)
//! und [`parse_asy`] (`.asy`-Bytes → JSON des [AsySymbol]-AST)
//! (Vertrag: `docs/documentation.md`, Abschnitt "WASM / Web-Export").
//! Fehler werfen den `Display`-String als JS-Wert; das Frontend fängt ihn
//! in `catch`.

use wasm_bindgen::prelude::*;

use crate::{asy_to_json, parse_asy_bytes, parse_bytes, schematic_to_json, AsySymbol, Schematic};

/// Parst `.asc`-Bytes und liefert das [crate::Schematic] als JSON-String.
///
/// Fehler (z. B. `line 3: invalid number …`) werden als `Display`-String
/// geworfen; Frontend: `catch (e) { message = String(e) }`.
#[wasm_bindgen]
pub fn parse(bytes: &[u8]) -> Result<String, JsValue> {
    let schematic: Schematic = parse_bytes(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    schematic_to_json(&schematic).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parst `.asy`-Bytes (LTspice-Symboleditor-Datei) und liefert das
/// [crate::AsySymbol] als JSON-String.
///
/// Fehler (z. B. `line 1: missing required Version line`) werden wie bei
/// [`parse`] als `Display`-String geworfen; Frontend: `catch (e) {
/// message = String(e) }`.
#[wasm_bindgen]
pub fn parse_asy(bytes: &[u8]) -> Result<String, JsValue> {
    let symbol: AsySymbol =
        parse_asy_bytes(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    asy_to_json(&symbol).map_err(|e| JsValue::from_str(&e.to_string()))
}
