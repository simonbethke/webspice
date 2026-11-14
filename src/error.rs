//! Fehler-Typ für das Lesen und Parcen von LTspice `.asc`-Dateien.

/// Fehler, die beim Lesen oder Parcen einer `.asc`- oder `.asy`-Datei
/// auftreten.
///
/// Alle Varianten, die sich auf eine Zeile beziehen, enthalten die
/// 1-basierte Zeilennummer; der `Display`-String beginnt dann mit `line <n>`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Ein unerwartetes Token wurde an einer Position gefunden, an der etwas
    /// anderes erwartet wurde.
    #[error("line {line}: unexpected token {found:?}, expected {expected:?}")]
    UnexpectedToken {
        /// Zeilennummer (1-basiert)
        line: usize,
        /// Erwarteter Wert bzw. Kontext (z. B. `Zeilenende`), frei formulierbar
        expected: String,
        /// Tatsächlich gefundener Token
        found: String,
    },

    /// Ein benötigtes Feld bzw. Token fehlt.
    #[error("line {line}: {token}: missing field {field:?}")]
    MissingField {
        /// Zeilennummer (1-basiert)
        line: usize,
        /// Betroffenes Zeilen-Keyword, z. B. `"WIRE"`
        token: &'static str,
        /// Name des fehlenden Feldes
        field: String,
    },

    /// Ein Zahlenwert war kein gültiger `i32`-Wert (oder außerhalb des Range).
    #[error("line {line}: invalid number {found:?} for field {field:?}")]
    InvalidNumber {
        /// Zeilennummer (1-basiert)
        line: usize,
        /// Name des Feldes
        field: String,
        /// Raw Token, der gefunden wurde
        found: String,
    },

    /// Ein Singleton-Token ist semantisch ungültig
    /// (Orientation, LineStyle, Alignment).
    #[error("line {line}: {token}: invalid {field} {found:?}")]
    InvalidToken {
        /// Zeilennummer (1-basiert)
        line: usize,
        /// Betroffenes Zeilen-Keyword
        token: &'static str,
        /// Feld, z. B. `"orientation"`
        field: String,
        /// Gefundener Token
        found: String,
    },

    /// `SYMATTR` ohne vorher aktives `SYMBOL`.
    #[error("line {line}: SYMATTR without a preceding SYMBOL")]
    OrphanedSymAttr {
        /// Zeilennummer (1-basiert)
        line: usize,
    },

    /// `.asy`: Die erste nicht-leere Zeile ist keine `Version`-Zeile
    /// (leere Datei: Zeile 1).
    #[error("line {line}: missing required Version line")]
    MissingVersion {
        /// Zeilennummer (1-basiert) der ersten nicht-leeren Zeile
        line: usize,
    },

    /// `.asy`: `PINATTR` ohne vorhergehendes `PIN`.
    #[error("line {line}: PINATTR without a preceding PIN")]
    OrphanedPinAttr {
        /// Zeilennummer (1-basiert)
        line: usize,
    },

    /// Dateisystem-Fehler beim Einlesen der Datei.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON-Serialisierungs-Fehler.
    ///
    /// In der Praxis nicht erreichbar (alle AST-Typen sind serialisierbar);
    /// vorhanden, damit `schematic_to_json` ohne `unwrap()` auskommen kann.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_line_number(err: &Error, line: usize) {
        let msg = err.to_string();
        assert!(
            msg.contains(&format!("line {line}")),
            "display must contain the line number, got: {msg:?}"
        );
    }

    /// Alle zeilenbezogenen Varianten zeigen die Zeilennummer.
    #[test]
    fn display_contains_line_numbers() {
        assert_line_number(
            &Error::UnexpectedToken {
                line: 12,
                expected: "end of line".into(),
                found: "x4".into(),
            },
            12,
        );
        assert_line_number(
            &Error::MissingField {
                line: 7,
                token: "WIRE",
                field: "x2".into(),
            },
            7,
        );
        assert_line_number(
            &Error::InvalidNumber {
                line: 3,
                field: "y1".into(),
                found: "abc".into(),
            },
            3,
        );
        assert_line_number(
            &Error::InvalidToken {
                line: 5,
                token: "SYMBOL",
                field: "orientation".into(),
                found: "R45".into(),
            },
            5,
        );
        assert_line_number(&Error::OrphanedSymAttr { line: 9 }, 9);
    }

    /// Varianten ohne Zeilennummer sind trotzdem sinnvoll darstellbar.
    #[test]
    fn non_line_variants_display() {
        assert!(!Error::Io(std::io::Error::other("boom"))
            .to_string()
            .is_empty());
        // `serde_json::Error` ist nicht direkt konstruierbar — aus ungültigem
        // JSON erzeugen.
        let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        assert!(Error::Json(serde_err).to_string().starts_with("json: "));
    }
}
