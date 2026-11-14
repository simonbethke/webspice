//! AST-Typen für LTspice-Schematics.
//!
//! Grundlage ist `ltspice_asc_format_specification.md` (Abschnitt 5);
//! die dort dokumentierten bewussten Abweichungen stehen im Abschnitt
//! "Model / AST" von `docs/documentation.md`.

use serde::Serialize;
use std::collections::HashMap;

/// Ein komplett geparstes LTspice-Schematic (`.asc`-Datei).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Schematic {
    /// Formatversion aus `VERSION`, z. B. `Some("4")`.
    pub version: Option<String>,
    /// Seiten-/Canvas-Definition aus `SHEET`.
    pub sheet: Option<Sheet>,
    /// Leitungen (`WIRE`).
    pub wires: Vec<Wire>,
    /// Named-Netze/Terminals (`FLAG`); `name == "0"` ist der Ground.
    pub flags: Vec<Flag>,
    /// Bus-Anschlusspunkte (`BUSTAP`).
    pub bustaps: Vec<Bustap>,
    /// Bauteil-Symbole inkl. ihrer `SYMATTR`-Attribute.
    pub symbols: Vec<Symbol>,
    /// Textblöcke (`TEXT`), inkl. SPICE-Direktiven und Kommentaren.
    pub texts: Vec<TextDirective>,
    /// Grafische (nicht-elektrische) Primitive.
    pub graphics: Vec<GraphicPrimitive>,
    /// Unbekannte Zeilen ohne offenes Symbol (roh, Roundtrip-Erhaltung).
    pub extras: Vec<String>,
}

/// Seiten-/Canvas-Definition (`SHEET`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sheet {
    /// Seitennummer.
    pub page: i32,
    /// Canvas-Breite.
    pub width: i32,
    /// Canvas-Höhe.
    pub height: i32,
}

/// Eine Leitung zwischen zwei Punkten (`WIRE`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Wire {
    /// Startpunkt X.
    pub x1: i32,
    /// Startpunkt Y.
    pub y1: i32,
    /// Endpunkt X.
    pub x2: i32,
    /// Endpunkt Y.
    pub y2: i32,
}

/// Named-Netz-Terminal (`FLAG`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Flag {
    /// Position X.
    pub x: i32,
    /// Position Y.
    pub y: i32,
    /// Name des Netzes; `"0"` ist der explizite Ground.
    pub name: String,
}

/// Bus-Anschlusspunkt (`BUSTAP`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bustap {
    /// Position X.
    pub x: i32,
    /// Position Y.
    pub y: i32,
}

/// Ein Bauteil-Symbol inkl. Attributen.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Symbol {
    /// Symbol-Typ (z. B. `res`, `cap`, `npn`).
    pub name: String,
    /// X der Ankerposition.
    pub x: i32,
    /// Y der Ankerposition.
    pub y: i32,
    /// Rotation / Spiegelung.
    pub orientation: Orientation,
    /// `SYMATTR`-Attribute (z. B. `InstName`, `Value`, `SpiceLine`);
    /// bei doppelmeldungen gilt der letzte Wert.
    pub attributes: HashMap<String, String>,
    /// Fenster-/Attribut-Anzeigeslots (`WINDOW`-Zeilen) im Symbol-Scope.
    pub windows: Vec<WindowSpec>,
    /// Unbekannte Zeilen im Symbol-Scope (roh, Roundtrip-Erhaltung).
    pub extras: Vec<String>,
}

/// Fenster-/Attribut-Anzeigeslot im Symbol-Scope (`WINDOW`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowSpec {
    /// Fenster-Index aus der `WINDOW`-Zeile.
    pub index: i32,
    /// X der Ankerposition.
    pub x: i32,
    /// Y der Ankerposition.
    pub y: i32,
    /// Attribut-Name, das das Fenster anzeigt.
    pub name: String,
    /// Sichtbarkeits-Flag wie in der Datei (`0`/`1`).
    pub visible: i32,
}

/// Rotation / Spiegelung eines Symbols (Spez. §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Orientation {
    /// Keine Rotation (0°).
    R0,
    /// 90° im Uhrzeigersinn.
    R90,
    /// 180°.
    R180,
    /// 270° im Uhrzeigersinn.
    R270,
    /// Horizontal gespiegelt, keine Rotation.
    M0,
    /// Horizontal gespiegelt, dann 90° im Uhrzeigersinn.
    M90,
    /// Horizontal gespiegelt, dann 180°.
    M180,
    /// Horizontal gespiegelt, dann 270° im Uhrzeigersinn.
    M270,
}

impl Orientation {
    /// Erzeugt eine [Orientation] aus einem Token (`R0`, `M270`, …).
    ///
    /// Liefert `None` für unbekannte Tokens.
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "R0" => Self::R0,
            "R90" => Self::R90,
            "R180" => Self::R180,
            "R270" => Self::R270,
            "M0" => Self::M0,
            "M90" => Self::M90,
            "M180" => Self::M180,
            "M270" => Self::M270,
            _ => return None,
        })
    }
}

/// Ein Textblock (`TEXT`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextDirective {
    /// Position X.
    pub x: i32,
    /// Position Y.
    pub y: i32,
    /// Ausrichtung (`Left`, `Center`, `Right`).
    pub align: String,
    /// Schriftgröße (typisch 1–4).
    pub size: i32,
    /// `true`, wenn der Text mit `!` beginnt (SPICE-Direktive).
    pub is_command: bool,
    /// Der vollständige Text inkl. `!`/`;`-Präfix.
    pub text: String,
}

impl LineStyle {
    /// Erzeugt einen [LineStyle] aus einem Token (`Normal`, `Bold`, …).
    ///
    /// Liefert `None` für unbekannte Tokens.
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "Normal" => Self::Normal,
            "Dotted" => Self::Dotted,
            "Dashed" => Self::Dashed,
            "Bold" => Self::Bold,
            _ => return None,
        })
    }
}

/// Strichstil für grafische Primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LineStyle {
    /// Durchgezogene, dünne Linie.
    Normal,
    /// Gestrichelte Linie.
    Dotted,
    /// Strichpunktierte Linie.
    Dashed,
    /// Durchgezogene, dicke Linie.
    Bold,
}

/// Grafische (nicht-elektrische) Primitive
/// (`LINE`, `RECTANGLE`, `CIRCLE`, `ARC`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GraphicPrimitive {
    /// Gerade Linie.
    Line {
        /// Strichstil.
        style: LineStyle,
        /// Startpunkt X.
        x1: i32,
        /// Startpunkt Y.
        y1: i32,
        /// Endpunkt X.
        x2: i32,
        /// Endpunkt Y.
        y2: i32,
        /// Optionales numerisches Schluss-Flag (z. B. `2`), `None` = nicht vorhanden.
        flag: Option<i32>,
    },
    /// Rechteck; Eckpunkte so wie in der Datei (keine Normalisierung,
    /// das bleibt Aufgabe des Renderer-Layers).
    Rectangle {
        /// Strichstil.
        style: LineStyle,
        /// Ecke 1 X.
        x1: i32,
        /// Ecke 1 Y.
        y1: i32,
        /// Ecke 2 X.
        x2: i32,
        /// Ecke 2 Y.
        y2: i32,
        /// Optionales numerisches Schluss-Flag (z. B. `2`), `None` = nicht vorhanden.
        flag: Option<i32>,
    },
    /// Kreis/Ellipse, angegeben durch Bounding-Box.
    Circle {
        /// Strichstil.
        style: LineStyle,
        /// Ecke 1 X.
        x1: i32,
        /// Ecke 1 Y.
        y1: i32,
        /// Ecke 2 X.
        x2: i32,
        /// Ecke 2 Y.
        y2: i32,
        /// Optionales numerisches Schluss-Flag (z. B. `4`), `None` = nicht vorhanden.
        flag: Option<i32>,
    },
    /// Elliptischer Bogen: Bounding-Box plus Start-/Endray-Punkte.
    Arc {
        /// Strichstil.
        style: LineStyle,
        /// Bounding-Box, Ecke 1 X.
        x1: i32,
        /// Bounding-Box, Ecke 1 Y.
        y1: i32,
        /// Bounding-Box, Ecke 2 X.
        x2: i32,
        /// Bounding-Box, Ecke 2 Y.
        y2: i32,
        /// Startray-Punkt X.
        x3: i32,
        /// Startray-Punkt Y.
        y3: i32,
        /// Endray-Punkt X.
        x4: i32,
        /// Endray-Punkt Y.
        y4: i32,
        /// Optionales numerisches Schluss-Flag, `None` = nicht vorhanden.
        flag: Option<i32>,
    },
}
