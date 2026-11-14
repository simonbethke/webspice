# webspice — Dokumentation

Single Source of Truth für die `webspice`-Library (LTspice `.asc`-Parser).

- Agenten-/Projektregeln: `AGENTS.md`
- Format-Spezifikationen (Referenz): `docs/spec/ltspice_asc_format_specification.md`
  (Sektionen 1–6), `docs/spec/ltspice_asy_specification.md`
- Task-Queue: `tasks/tasklog.md`

## Purpose & Scope
`webspice` liest LTspice-Schematic-Dateien (`.asc`, UTF-8 oder Windows-1252) ein und liefert
das strukturierte AST `Schematic` (siehe Abschnitt "Model / AST").

- Im Scope: Zeilen-Parsing, AST, Fehler-Reporting (mit Zeilennummern);
  Rendering läuft im Web-Frontend (siehe „Web-Frontend (MVP)")
- **Nicht** im Scope: CLI, Netlist-Erzeugung, Simulation
- Aktueller Stand: **Tasks 01–12 + 14 abgeschlossen** (Parsing, Robustheit,
  WASM, Web-Frontend MVP, `.asy`-Symbol-Rendering, LTspice-Optik,
  LTspice-Referenz-Korrekturen, LTspice-Abgleich 2, GitHub-Pages-Deployment);
  Backlog (unnummeriert, nicht als Nächstes): Build+Deploy via
  GitHub Actions — s. Tasklog.

## Projektstruktur
- `src/` — Rust-Library (Parser, AST, Fehler); `src/wasm.rs` = WASM-Export
- `tests/fixtures/*.asc`, `tests/asy-fixtures/*.asy` — Integrations-Fixtures
- `sym/` — LTspice-Original-Symbolkorpus (~4800 `.asy`), nur lesen
- `web/` — Web-Frontend (Stencil + TypeScript), siehe „Web-Frontend (MVP)“ → „Struktur“
- `docs/documentation.md` — diese Datei; `docs/spec/` — Format-Spezifikationen
- `tasks/` — Task-Queue (`tasklog.md` + eine Datei je offenem Task)
- `AGENTS.md` — Agentenregeln; `README.md`

## API

| Funktion | Zweck |
|---|---|
| `parse_file(path) -> Result<Schematic, Error>` | liest eine Datei ein und parst sie |
| `parse_bytes(data: &[u8]) -> Result<Schematic, Error>` | parst einen Byte-Buffer |
| `parse_reader(reader: impl BufRead) -> Result<Schematic, Error>` | parst aus einem Strom |
| `schematic_to_json(&Schematic) -> Result<String, Error>` | serialisiert das AST als JSON-String (Web-Export, siehe „WASM / Web-Export") |
| `parse_asy_bytes(data: &[u8]) -> Result<AsySymbol, Error>` | parst eine LTspice-Symboldefinition (`.asy`; siehe „Symbol-Rendering (.asy)") |
| `asy_to_json(&AsySymbol) -> Result<String, Error>` | serialisiert `AsySymbol` als JSON-String (Web-Export) |

- Zeilen-Nummern in Fehlern sind **1-basiert**
- Leere Zeilen werden übersprungen; unbekannte Tokens (z. B. `IOPIN`, `DATAFLAG`)
  werden roh erhalten (Roundtrip) und schließen ein offenes Symbol nicht
  (Forward-Kompatibilität)
- CRLF und LF sind gleichwertig; Argumente sind durch beliebige
  ASCII-Leerzeichen getrennt (auch Tabs)
- Ein UTF-8-BOM am Datei-Anfang wird ignoriert
- Nicht-UTF-8-Eingabe (z. B. Windows-1252 `0xB5` = µ — LTspice-Standard unter
  Windows) wird byte-für-Byte **verlustfrei als Windows-1252 (cp1252) nach
  Unicode** dekodiert und parst weiter ohne Fehler; gültiges UTF-8 bleibt
  unverändert (0x80–0x9F nach cp1252-Tabelle, undefinierte Einträge 0x81, 0x8D,
  0x8F, 0x90, 0x9D mappen verlustfrei auf C1-Kontrollzeichen). Der Fallback
  betrifft die Gesamteingabe, nicht einzelne Bytes
- Keine `unsafe`, keine `unwrap()`/`expect()` außer in Tests

## Format-Referenz
Keyword = erster Token der Zeile, groß geschrieben, case-sensitive:

| Token | Form | Ziel-Feld |
|---|---|---|
| `VERSION` | `<major[.minor]>` | `Schematic.version` |
| `SHEET` | `<page> <width> <height>` | `Schematic.sheet` |
| `WIRE` | `<x1> <y1> <x2> <y2>` | `Schematic.wires` |
| `FLAG` | `<x> <y> <net>` (`0` = Ground) | `Schematic.flags` |
| `BUSTAP` | `<x> <y>` | `Schematic.bustaps` |
| `SYMBOL` | `<type> <x> <y> <orientation>` | `Schematic.symbols` |
| `SYMATTR` | `<key> <value…>` | `Symbol.attributes` |
| `WINDOW` | `<index> <x> <y> <attr-Name> <visible>` | `Symbol.windows` |
| `TEXT` | `<x> <y> <align> <size> <content…>` | `Schematic.texts` |
| `LINE` / `RECTANGLE` / `CIRCLE` | `<style> <x1> <y1> <x2> <y2> [flag]` | `Schematic.graphics` |
| `ARC` | `<style> <x1>…<y4>` (8 Koordinaten) `[flag]` | `Schematic.graphics` |

- Koordinaten: `i32`; Ursprung oben-links, `+X` rechts, `+Y` unten
- Orientation-Tokens: `R0 R90 R180 R270 M0 M90 M180 M270`
- Style-Tokens: `Normal Dotted Dashed Bold`
- Alignment-Tokens: `Left Center Right`
- Zu wenig Argumente → `MissingField`; zu viele → `UnexpectedToken`
  (beide mit Zeilennummer)
- `LINE`/`RECTANGLE`/`CIRCLE`/`ARC`: höchstens ein optionales numerisches
  Schluss-Flag (`<flag>`, z. B. `2` oder `4`) hinter den Koordinaten →
  `flag: Option<i32>` (Bedeutung wird nicht interpretiert); kein Flag = `None`
  (echte LTspice-Dateien schreiben diese Flags); zweites zusätzliches Token →
  `UnexpectedToken`, nicht-numerisches Flag → `InvalidNumber`
- `SHEET`: `page >= 1`, `width`/`height` nicht-negativ (sonst `InvalidNumber`)
- `TEXT`: `!`-Präfix = SPICE-Direktive → `is_command = true`; `;`-Präfix = Kommentar;
  Freitext → `is_command = false`. Das Präfix bleibt immer im `text`-Feld enthalten.
  `size >= 1` (sonst `InvalidNumber`)
- `TEXT`-Inhalt = Rest der Zeile hinter `size`, verbatim (vorne/hinten getrimmt,
  innere Leerzeichen bleiben); daher darf `TEXT` mehr als 5 Argumente haben
  (kein "zu viele"-Fehler, ungleich zu den Token mit fester Argumentanzahl)
- `SYMATTR`-Werte dürfen Leerzeichen enthalten: Wert = Rest der Zeile hinter dem
  Key, verbatim (vorne/hinten getrimmt, innere Leerzeichen bleiben)
- `SYMATTR` gehört immer zum zuletzt deklarierten `SYMBOL`; nur ein
  bekanntes Nicht-SYMATTR-Keyword (SYMBOL, VERSION, SHEET, WIRE, FLAG,
  BUSTAP, TEXT, LINE, RECTANGLE, CIRCLE, ARC) oder das Datei-Ende schließt
  das offene Symbol (Flush); `WINDOW`-Zeilen und unbekannte Tokens bleiben
  im Symbol-Scope
- `WINDOW` im Symbol-Scope → typisiert `Symbol.windows`; `WINDOW` außerhalb
  des Scopes oder mit abweichender Struktur (≠ 5 Felder) bleibt roh in
  `extras`
- Unbekannte Tokens im Symbol-Scope → roh in `Symbol.extras`; ohne offenes
  Symbol → roh in `Schematic.extras` (Roundtrip-Erhaltung)
- `SYMATTR` ohne aktives SYMBOL → `OrphanedSymAttr`
- Duplizierte `SYMATTR`-Keys: letzter Wert gewinnt

## Model / AST
- `Schematic { version: Option<String>, sheet: Option<Sheet>, wires, flags, bustaps, symbols, texts, graphics, extras }`
- `Sheet { page, width, height }`, `Wire { x1, y1, x2, y2 }`
- `Flag { x, y, name }`, `Bustap { x, y }`
- `Symbol { name, x, y, orientation: Orientation, attributes: HashMap<String, String>,
  windows: Vec<WindowSpec>, extras: Vec<String> }`
  (`attributes` = SYMATTR Key→Wert, letzter Wert gewinnt)
- `WindowSpec { index: i32, x: i32, y: i32, name: String, visible: i32 }`
  (= eine `WINDOW`-Zeile)
- `Symbol.extras` / `Schematic.extras` = unbekannte Zeilen, roh (Roundtrip-
  Erhaltung; `Symbol.extras` im Symbol-Scope, `Schematic.extras` ohne offenes Symbol)
- `TextDirective { x, y, align: String, size: i32, is_command: bool, text: String }`
- `GraphicPrimitive::{Line, Rectangle, Circle, Arc}` — alle mit `style: LineStyle` + Koordinaten
  + `flag: Option<i32>` (optionales numerisches Schluss-Flag; `None` = nicht vorhanden)
- `Orientation::{R0, R90, R180, R270, M0, M90, M180, M270}` (mit `from_token`)
- `LineStyle::{Normal, Dotted, Dashed, Bold}` (mit `from_token`)
- alle Typen: `Debug, Clone, PartialEq`

**Bewusste Abweichungen gegenüber dem AST-Blueprint der Spez. (§5):**
1. `Symbol.orientation` ist ein `Orientation`-Enum statt `String`
   — typsicher und erschöpft die 8 gültigen Tokens.
2. `Schematic.bustaps: Vec<Bustap>` — der AST-Blueprint der Spez. hat kein
   `BUSTAP`-Feld, aber §3.2 definiert das Token; die Daten müssen ablegbar sein.

## Errors
`Error`-Varianten (alle zeilenbezogenen haben 1-basierte Zeilennummer,
`Display` beginnt dann mit `line <n>`):

| Variante | Bedeutung |
|---|---|
| `UnexpectedToken { line, expected, found }` | unerwartetes Token an einer Position |
| `MissingField { line, token, field }` | benötigtes Field/Token fehlt |
| `InvalidNumber { line, field, found }` | Zahl nicht valide / außerhalb des Range |
| `InvalidToken { line, token, field, found }` | Singleton-Token ungültig (Orientation, Style, Alignment) |
| `OrphanedSymAttr { line }` | SYMATTR ohne zuvor aktives SYMBOL |
| `MissingVersion { line }` | `.asy`: die erste nicht-leere Zeile ist keine `Version`-Zeile |
| `OrphanedPinAttr { line }` | `.asy`: `PINATTR` ohne vorhergehendes `PIN` |
| `Io(std::io::Error)` | Datei-/Stream-Fehler |

Beispiel: `Error::InvalidNumber { line: 3, field: "y1", found: "abc" }`
→ `line 3: invalid number "abc" for field "y1"`

## Symbol-Rendering (.asy)

### Überblick

Symboldefinitionen liegen bei LTspice als separate ASCII-Dateien `.asy`
(vorhanden als Korpus mit ~4800 Dateien unter `sym/`). Webspice parst sie
in `AsySymbol` und zeichnet damit **echte Symbol-Geometrie** statt der
Platzhalter-Boxen aus
Task 08 auf der Platine. Format-Basis ist
`docs/spec/ltspice_asy_specification.md`; wo die reale Korpus-Praxis über die
Spezifikation hinausgeht (z. B. `PINNAME`-Formen wie `+`, `-`, `NC+`),
gilt das in dieser Sektion Beschriebene.

### Parsing (`parse_asy_bytes`)

- Schlüsselwörter sind **case-insensitiv**.
- Die erste nicht-leere Zeile **muss** `Version …` sein → sonst
  `MissingVersion { line }` (leere Datei: `line 1`).
- Zeilentypen (andere werden **stillschweigend ignoriert** — keine
  Roundtrip-Variante für `.asy`):
  - `SYMBOLTYPE <token>`: nur `CELL` oder `BLOCK` (case-insensitiv,
    verbatim gespeichert), sonst `InvalidToken` (Korpus: 4126× `CELL` /
    581× `BLOCK`).
  - `SYMATTR <key> <value…>`: `AsySymbol.properties[key] = value` verbatim,
    Duplikate: letzter gewinnt (wie `.asc`).
  - `WINDOW <index> <x> <y> <just> <n>` (exakt 5 Argumente, wie `.asc`):
    → `WindowSpec { index, x, y, name: <just>, visible: <n> }` in
    `AsySymbol.windows`; `<just>` verbatim (Korpus: `Left`, `Center`,
    `Top`, `Bottom`, `Right`), `<n>` Integer verbatim (Korpus: `0`–`5`);
    fehlendes Argument → `MissingField`, zusätzliches →
    `UnexpectedToken`, ungültige Zahlen → `InvalidNumber`.
  - `LINE|RECTANGLE|CIRCLE <style> x1 y1 x2 y2 [width]`: →
    `GraphicPrimitive` (gleiche Form wie in `.asc`); `ARC <style>` + 3
    Koordinatenpaare. Stil-Tokens: `Normal` und `Wide` (→
    `LineStyle::Bold`, d. h. `stroke-width:2` im Web); `Dotted`/`Dashed`
    werden in `.asy` abgelehnt (`LineStyle`-Fehler → `InvalidToken`).
    Zu wenige Argumente → `MissingField`, zu viele → `UnexpectedToken`
    (je mit Zeilennummer).
  - `TEXT <x> <y> <just> <n> <text…>`: → `AsyText { x, y, justification,
    text }`; `text` bis Zeilenende verbatim (inkl. `!`/`;`-Präfix); die
    numerische Größe `<n>` wird gelesen, aber nicht abgelegt
    (JSON-Vertrag aus der Spezifikation). Tolerierte Kurzform `<x> <y>
    <text…>` ohne just/n (eine 4-Argumente-Zeile zählt als Kurzform, falls
    das 4. Argument keine Zahl ist — sonst als Volle Form ohne Text →
    `MissingField`).
  - `PIN <x> <y> <just> <offset> [name]` (exakt 4 oder 5 Argumente): →
    `AsyPin { x, y, justification, offset, name, properties }`; die
    Justification ist ein freier String (Korpus nutzt `NONE`); `name` verbatim (auch `+`,
    `-`, `NC+`). Zu wenige Argumente → `MissingField`; nicht-numerische Koordinate/`offset` → `InvalidNumber` (`field` = Argumentname).
  - `PINATTR <key> <value…>`: bindet an den **letzten** `PIN` →
    `AsyPin.properties[key]` (Duplikate: letzter gewinnt); ohne
    vorausgehendes `PIN` → `OrphanedPinAttr { line }`.
- Neue `Error`-Varianten für `.asy`: `MissingVersion { line }`,
  `OrphanedPinAttr { line }` (Display beginnt wie alle Zeilenbezogenen mit
  `line <n>`); `InvalidToken`, `MissingField`, `UnexpectedToken` und
  `InvalidNumber` werden mit Zeilennummer wiederverwendet.

### Auflösung & Rendering (Web)

- **Schlüssel:** Name des `.asc`-`SYMBOL` ↔ **Dateiname (Basename, ohne
  Endung, case-insensiv)** der `.asy`.
- **Priorität:** (1) vom Nutzer hochgeladene `.asy` (Filepicker),
  (2) statische Kurzbibliothek `web/webspice-symbols/*.asy` (committed;
  Standard-Bibliothek = `sym/`-Originale: `res` (europäisches
  Rechteck, byte-identisch zu `EuropeanResistor.asy`/`sym/res.asy`),
  `cap`, `ind`, `diode`, `voltage`, `current`; dazu jtk3-Einträge
  `pot`, `pot_log`, `EuropeanPolcap`, `EuropeanResistor`, `triode`,
  `npn`, `LED`, `nmos`, `pjf`, `spdt`, `spst`, `zener`, `ind2`;
  `sym/` hat **kein** `gnd.asy` → FLAG `0` bleibt code-gezeichnetes
  Erd-Symbol), (3) Platzhalter-Box (Task 08).
- **Geometrie:** im Local-Frame des `SYMBOL` — `translate(x y)` plus
  Orientation-Transform (dasselbe Mapping für aufgelöste `.asy`-Symbole
  und Platzhalter-Box; empirisch abgeleitet und per Pin↔Wire-Endpunkte
  in `jtk3.asc` verifiziert, Bugfix 2026-11-13): `R0` → keins,
  `R90` → `rotate(90)`, `R180` → `rotate(180)`, `R270` → `rotate(270)`,
  `M0` → `scale(-1 1)`, `M90` → `rotate(-90) scale(-1 1)`,
  `M180` → `rotate(-180) scale(-1 1)`, `M270` → `rotate(-270) scale(-1 1)`
  — d. h. `Mθ` = Spiegelung an der lokalen y-Achse gefolgt von
  Gegenrotation; lokale Koordinaten: `M0` `(x,y)→(−x,y)`,
  `M90` `(x,y)→(y,x)`, `M180` `(x,y)→(x,−y)`, `M270` `(x,y)→(−y,−x)`.
  `viewBox` = Bounding-Box des Contents (Task 10): FLAG `0`
   (Erd-Symbol) erweitert unten um +18 LE (Stangenlänge); der
   Blatt-Rahmen (Sheet) bleibt bewusst **außerhalb** der viewBox
   (die SHEET-Größe würde sonst den Content überlagern —
   dokumentierte Ausnahme).
- Instanz-`WINDOW`-Positionen (World) werden zusätzlich
  in die Bounding-Box eingerechnet (Task 11: Window-Labels
  nicht clippen).
- **Upright-Text (Task 10):** alle Text-Elemente (`.asy`-`TEXT`,
  `Value`, `InstName`, Fallback-Box-Label) bleiben bei jeder
  Orientation lesbar: Text-Wrapper-`<g>` = `translate(lx ly)` +
  Gegenrotation (Inverses der Orientation-Rotation; Spiegelungen
  ohne Rotation): `R0` → keins, `R90` → `rotate(-90)`,
  `R180` → `rotate(-180)`, `R270` → `rotate(-270)`, `M0` →
  `scale(-1 1)`, `M90` → `matrix(0 1 1 0 0 0)`, `M180` → `scale(1 -1)`,
  `M270` → `matrix(0 -1 -1 0 0 0)`. Vertikale `.asy`-`TEXT`
  (`*V`-Justification) kombinieren Gegenrotation mit
  `rotate(-90)`. Grafik und Pins bleiben im Orientation-Transform
  (Geometrie-Punkt oben; Fix 2026-11-13 gültig).
- **Render-Konfiguration (Task 10):** alle variablen
  Rendering-Werte (Schrift, -größe, -fett, Strichbreite, Farben,
  Ansichts-Flags) kommen aus **einer** Config-Datei
  (`web/src/render/render-config.json`; TS-Interface `RenderConfig`
  in `render.ts`). `renderSchematic(ast, host, symbols?, config?)` —
  optionaler letzter Parameter, Default = JSON-Import (dafür
  `resolveJsonModule: true` in `web/tsconfig.json`); `render.ts`
  bleibt pure; Tests dürfen Ad-hoc-Configs übergeben. Mapping:
  Wires + Blatt-Rahmen → `wireColor`; `.asy`-Grafik, `.asc`-Grafik,
  Fallback-Box-Rahmen, Erd-Symbol → `symbolColor`;
  `Value`/`InstName`, `.asy`-`TEXT`, `.asc`-`TEXT` → `textColor`;
  Netlabels → `netlabelColor`; `BUSTAP` → `pinColor`;
  Grid-Punkte → `gridPointColor`; Unverdrahtete-Pin-Marker →
  `unconnectedPinColor`; Strich: `penThickness` (Bold = 2×).
- **Schrift/TEXT-Größen (Task 10, Gewicht Task 11):** alle Texte:
  `font-family` und `font-weight` (nur wenn `fontBold`) aus der Config
  (ersetzt `monospace`). **Gewicht (Task 11):** `InstName`, `Value`,
  `.asc`-`TEXT` → bold laut Config; **`SpiceLine` und `.asy`-`TEXT` →
  non-bold** (Override). `.asc`-`TEXT`: deterministische Regel
  `fontSize·size/3`, gerundet auf 1 Dezimale (Default: size 4 →
  13.3 > Bauteil-Label 10 > size 2 → 6.7); alle anderen Texte:
  `fontSize`. **Präfix-Strip (Task 11):** `.asc`-`TEXT` beginnt mit `;`
  oder `!` → erstes Zeichen wird bei der Darstellung gestrippt (AST und
  `is_command` bleiben unverändert; Zeilen ohne Präfix rendern unverändert).
- **Labels via WINDOW-Slots (Task 11):** InstName/Value/SpiceLine werden
  in der Position des `WINDOW`-Slots platziert (Rust-Parser
  (`.asc`) und `.asy`-Parser liefern beide `windows`):
  Slot `0` → `attributes['InstName'] ?? '?'` — **immer** gerendert
  (außer `Invisible`), ohne Window: Default `(0,0)`, `start`;
  Slot `3` → `attributes['Value'] ?? asy SYMATTR Value` — **nur wenn
  Window 3 existiert** (und nicht `Invisible`), sonst übersprungen,
  **kein Fallback-Render**; Slot `39` →
  `attributes['SpiceLine'] ?? asy SYMATTR SpiceLine` — **nur wenn
  Window 39 existiert** (und nicht `Invisible`), sonst übersprungen,
  **kein Fallback-Render**. Auflösung pro Slot (höchste zuerst):
  Instanz-`WINDOW` im `.asc` → `WINDOW` im `.asy` → Renderer-Default.
  Andere Slot-Indizes werden ignoriert; das letzte `WINDOW`-Feld
  (`1`/`2`) ist der LTspice-Gruppendifferenzierer (kein
  Rendering-Effekt). **Justification (edge-Modell, Task 14):**
  Plain-Horizontal-Tokens benennen die rechte/linke Textkante am
  Anker, `V`-Tokens die Welt-Kante des Textblocks am Anker — beide
  decken sich bei `Left`/`Right`: `Left` → `text-anchor: end` (rechte
  Textkante am Anker, Text wächst nach links), `VLeft` → `start`,
  `Right` → `start` (linke Textkante am Anker, Text wächst nach
  rechts), `VRight` → `end`, `Center`/`VCenter` → `middle`; alle
  Horizontal-Tokens `dominant-baseline: central`. Vertikal:
  `Top`/`VTop` → `middle` + `text-after-edge` (Text **über** dem
  Anker), `Bottom`/`VBottom` → `middle` + `text-before-edge` (Text
  **unter** dem Anker); `Invisible` → Slot-Label wird nicht
  gerendert (kein Fallback). **keine Rotation** (Rotation gilt nur
  für `.asy`-`TEXT` mit `*V`-Justification); 90°/270°-gedrehte
  Bauteile rendern InstName/Value **horizontal** (nur
  Position/Justierung/Größe/Farbe, keine Textrichtung).
  Platzhalter-Box bleibt nur Fallback; alle Labels **upright**
  (Upright-Text-Punkt oben) in `textColor`.
- **Pins:** keine Pin-Dots bei normaler Darstellung; Pins fließen
  nur in die Junction- und Unverdrahtet-Regel ein (Pin-Endpunkte
  über das Orientation-Mapping in Weltkoordinaten, s. Geometrie).
- **TEXT:** `.asy`-`TEXT` wird im Local-Frame bei `(x, y)` mit
  LTspice-Anchor-Mapping gerendert (`Left*` → `start`, `Center*` →
  `middle`, `Right*` → `end`; `*V` → vertikale Verschiebung,
  sonst vertikale Zentrierung) — **upright** (Gegenrotation;
  `*V` zusätzlich `rotate(-90)`) in `textColor`.
- **Flags (Task 10):** FLAG `0` → LTspice-Erd-Symbol (vertikale
  Linie + drei abnehmende waagrechte Balken, Stange Länge 18,
  Balken-Halbweiten 9/6/3) in `symbolColor`; alle anderen
  FLAG-Namen → **Netlabel**: Net-Name als `<text>` in
  `netlabelColor`.
- **Grid-Punkte (Task 11, Größe Task 14):** Toggle `showGridPoints` — gefüllte
  `gridPointSize`×`gridPointSize`-Quadrate (Default 6, config
  `gridPointSize`) in `gridPointColor` **nur an echten Junctions**:
  > 2 Verbindungsenden am exakten Punkt; Verbindungsenden = beide
  Wire-Endpunkte + aufgelöste Pin-World-Enden (Pins über das
  Orientation-Mapping); 2-Draht-Ecke und Pin+Draht (je 2 Enden) →
  **kein** Punkt; Flag/BUSTAP/Text werden nicht gezählt.
- **Unverdrahtete Pins (Task 11):** Toggle `markUnconnectedPins` —
  gefüllte Kreise (Radius 2) in `unconnectedPinColor` an jedem Pin-Endpunkt eines
  aufgelösten `.asy`-Symbols, an dem kein Wire-Endpunkt innerhalb
  von ≤ 1 LE liegt (Pin-Koordinate über das Orientation-Mapping
  aus dem Geometrie-Punkt in Weltkoordinaten).
- **Blatt-Rahmen (Task 10):** Toggle `showTitleBlocks` +
  vorhandenem SHEET → `<rect>` (SHEET-Breite × -Höhe) bei (0,0)
  in `wireColor`, hinten gezeichnet, **nicht** in der viewBox
  (Ausnahme, s. Geometrie-Punkt).
- **Out of Scope** (aufgabe-spezifisch): `SYMKEY`, Parameter-Bindung,
  Window-Slots außerhalb {0,3,39} (andere Slots werden ignoriert),
   vollständige `DRAW`-Varianten,
  Symbol-Editor, `.lib`/`.inc`-Parsing.

### JSON-Format `AsySymbol` (WASM-Vertrag)

- `parse_asy(bytes) → String` (JSON von `AsySymbol`); Fehlerbehandlung wie
  bei `parse()` (`throw` mit dem `Display`-String):
  `{ version, symbolType, properties{}, windows[], graphics[], texts[],
    pins[] }` mit
  - `window`: `WindowSpec` (siehe `.asc`-Format),
  - `graphic`: `GraphicPrimitive` (siehe `.asc`-Format),
  - `text`: `{ x, y, justification, text }`,
  - `pin`: `{ x, y, justification, offset, name?, properties{} }`
    (`name` fehlt, wenn die `PIN`-Zeile keinen Namen hatte).

## WASM / Web-Export
Browser-Export der Library (wasm32), vom Web-Frontend (Abschnitt
„Web-Frontend (MVP)") konsumiert:

- Target `wasm32-unknown-unknown`; Build:
  `npm run build-wasm` in `web/` (wasm-pack + `normalize-wasm.cjs`,
  Output `web/wasm/`, siehe Abschnitt „Web-Frontend (MVP)")
  (wasm-pack ≥ 0.14; `rustup target add wasm32-unknown-unknown` nötig)
- `[lib] crate-type = ["cdylib", "rlib"]` (wasm-Export benötigt `cdylib`;
  `rlib` bleibt für Rust-Konsumenten)
- Dependencies: `serde` (derive) + `serde_json` (host & wasm);
  `wasm-bindgen` **nur** im
  `[target.'cfg(target_arch = "wasm32")'.dependencies]`
- WASM-Export: `parse(bytes: &[u8]) -> Result<String, Error>` (`src/wasm.rs`)
  = `parse_bytes` + `schematic_to_json`
- WASM-Export: `parse_asy(bytes: &[u8]) -> Result<String, Error>`
  (`src/wasm.rs`) = `parse_asy_bytes` + `asy_to_json`
  (JSON-Vertrag: Abschnitt „Symbol-Rendering (.asy)")
- **JSON-Vertrag** (Single Source of Truth; `web/wasm/wasm.d.ts` trägt nur die
  Funktion-Signatur, die AST-Typen spiegelt `web/src/types/ast.ts`):
  - Tasten = exakte Rust-Feldnamen, snake_case;
    Optionale `version`/`sheet` = `null` bei Abwesenheit
  - Unit-Enums (`orientation`, `style`) = Token-String (`"R90"`, `"Dashed"`)
  - `Symbol.attributes` = JSON-Objekt (Key→Wert)
  - `GraphicPrimitive` = `{"Line"|"Rectangle"|"Circle"|"Arc": {…Felder}}`;
    alle Varianten tragen `flag: i32 | null`
  - Zahlen = i32
  - `Symbol.windows` = Array von `{index, x, y, name: String, visible}`;
    `Symbol.extras` / `Schematic.extras` = Array von Rohzeilen (String)
- **Fehler-Vertrag**: `Err` wird als `Display`-String als JS-Wert geworfen
  (z. B. `line 3: invalid number …`); Frontend fängt in `catch (e) {
  message = String(e) }`. `parse_file` bleibt host-only.

## Web-Frontend (MVP)
Stencil v4 + TypeScript in `web/`; Build-Output `web/www/`
(GH-Pages-tauglich, Task 12). `index.html` mountet `<app-root>`, der das
Element `<schematic-viewer>` trägt; das Rendern läuft über die pure Funktion
`renderSchematic(ast, host, symbols?, config?)` (SVG-DOM, testbar ohne
Browser; Theme/Typografie/Toggles aus `render-config.json`, siehe
„Render-Konfiguration" unter „Symbol-Rendering (.asy)").

### Struktur
```
web/
├── package.json  stencil.config.ts  tsconfig.json
├── index.html                          # mountet <app-root>
├── normalize-wasm.cjs                  # WASM-Build → web/wasm/ normalisieren
├── copy-index.cjs                      # index.html → web/www/ kopieren (nach stencil build)
├── wasm/                               # normalisiertes WASM-Paket (committed)
├── webspice-symbols/                   # statische Kurzbibliothek (committed, `.asy`)
└── src/
    ├── app.tsx                         # <app-root> → <schematic-viewer>
    ├── wasm.ts                         # lazy WASM-Loader (einmalig, gecacht)
    ├── types/ast.ts                    # TS-Interfaces = JSON-Vertrag (Abschnitt „WASM / Web-Export")
    ├── render/
    │   ├── render-config.json          # LTspice-Theme/-Toggles (JSON, Task 10; Default-Config für render.ts)
    │   ├── render.ts                   # PURE: AST → SVG-DOM
    │   └── render.spec.ts              # jsdom-Unit-Tests
    └── components/core/schematic-viewer/
        ├── schematic-viewer.tsx        # Filepicker, DropZone, Status, Error-Panel, Pan/Zoom
        └── schematic-viewer.css
```

### Verhalten `<schematic-viewer>`
- Filepicker (`accept=".asc"`) + DropZone; Laden:
  `file.arrayBuffer()` → `Uint8Array` → WASM `parse()` → `JSON.parse`
- `.asy`-Symbolauflösung (Abschnitt „Symbol-Rendering (.asy)"): beim Start
  werden die Dateien der statischen Kurzbibliothek `webspice-symbols/`
  via WASM `parse_asy()` geladen und geparst; eine per
  Filepicker/Drop geladene `.asy` ersetzt den Eintrag mit demselben
  Namen (höhere Priorität); nicht auflösbare `SYMBOL`s bleiben
  Platzhalter-Boxen
- Statuszeile: Dateiname, Bytes, Parse-Dauer (ms),
  Zählungen wires/flags/bustaps/symbols/texts/graphics, zusätzlich
  Symbol-Auflösung: `<aufgelöst>/<gesamt>`
- Fehler-Panel: roter Block mit `String(e)` (geworfener String enthält
  `line N: …`, siehe Fehler-Vertrag in „WASM / Web-Export");
  das zuletzt erfolgreiche Rendering bleibt sichtbar
- `viewBox` = BoundingBox aller Elemente ± 10 px; ohne Elemente =
  SHEET-Größe; ohne SHEET = 100×100; dazu FLAG `0` (Erd-Symbol) +18
  LE unten, Sheet-Rahmen ausgenommen (Task 10, s. „Geometrie")
- Pan/Zoom: Pointer-Drag, Wheel um den Cursor, Buttons `+`, `−`, `1:1`,
  `Fit`; umgesetzt als `transform: translate(…) scale(k)` auf
  `<g class="viewport">` (k geklemmt auf 0.1 … 50)

### AST → SVG-Mapping
| AST-Element | SVG |
|---|---|
| `wire` | `<line>` stroke `wireColor`, `stroke-width` = `penThickness` |
| `flag` | Name `0` → Erd-Symbol (vertikale Stange + drei abnehmende Balken) in `symbolColor`, `penThickness`; sonst **Netlabel**: `<text>` mit Net-Name in `netlabelColor`, Schrift laut „Render-Konfiguration" |
| `bustap` | `<rect>` 4×4 gefüllt `pinColor` |
| `text` | `<text>` bei (x,y); `font-family`/`-weight`/`-size` aus der Config; `.asc`-`TEXT`: `font-size` = `fontSize·size/3` (1 Dezimale; size 4 klar größer, size 2 klar kleiner als Bauteil-Labels); Fill `textColor`; `align` (`Left`/`Center`/`Right`) → `text-anchor` (`start`/`middle`/`end`); **Gewicht (Task 11):** `.asc`-`TEXT` bold laut Config, `.asy`-`TEXT` non-bold; **Prefix-Strip (Task 11):** `.asc`-`TEXT` mit führendem `;`/`!` → erstes Zeichen wird bei der Anzeige gestrippt (AST/`is_command` unverändert) |
| `line` (Graphic) | `<line>` stroke `symbolColor`; `Bold` → `stroke-width: penThickness·2`, `Dashed` → `stroke-dasharray:6 3`, `Dotted` → `stroke-dasharray:2 2`, `Normal`/unbekannt → `penThickness` |
| `rectangle` | `<rect x=min(x1,x2) y=min(y1,y2) w=\|x2-x1\| h=\|y2-y1\|>`, ohne Füllung |
| `circle` | `<ellipse cx=(x1+x2)/2 cy=(y1+y2)/2 rx=\|x2-x1\|/2 ry=\|y2-y1\|/2>` |
| `arc` | `<path>` mit `A`-Kommando von (x3,y3)→(x4,y4), Ellipsen-Achsen aus BBox (K1–K4); **dokumentierte Näherung** (`large-arc=0`, `sweep=1`; LTspice-Bogenrichtung nicht 1:1 reproduzierbar) |
| `symbol` (aufgelöst) | `<g translate(x,y) [Orientation-Transform]>` mit dem Transform laut „Geometrie"-Punkt unter „Auflösung & Rendering (Web)" + Geometrie der auflösenden `.asy`: Primitive wie `line`/`rectangle`/`circle`/`arc` (Style `Bold` = `Wide` → `penThickness·2`), stroke `symbolColor`; keine Pin-Punkte; `.asy`-`TEXT` wie `text` (Anchor-Mapping, **upright**, `textColor`); Labels laut Bullet „**Labels via WINDOW-Slots (Task 11)**“ (Slot 0/3/39, `.asy`-Fallback, Skip-Regeln, Window-Justification, SpiceLine non-bold) — **upright** |
| `symbol` (Fallback) | Placeholder: `<g translate(x,y) [Orientation-Transform]>` mit `<rect 32×14>` zentriert, Rahmen `symbolColor`; Label = `InstName` (aus `attributes`), sonst `Value`, sonst `?` — **upright**; Orientation-Transform laut „Geometrie"-Punkt — nur wenn keine `.asy` auflösbar |
| `grid-points` (Task 11) | gefüllte `gridPointSize`×`gridPointSize`-Quadrate (Default 6) in `gridPointColor` **nur an echten Junctions** (> 2 Verbindungsenden am exakten Punkt: Wire-Enden + aufgelöste Pin-World-Enden) — wenn `showGridPoints` |
| `unconnected-pins` (Task 11) | gefüllte Kreise (Radius 2) `unconnectedPinColor` an Pin-Endpunkten (Weltkoordinaten) ohne Wire-Endpunkt ≤ 1 LE — wenn `markUnconnectedPins` |
| `sheet-frame` (Task 10) | `<rect>` SHEET-Breite × -Höhe bei (0,0), stroke `wireColor`, `penThickness`, hinten, **nicht** in der viewBox — wenn `showTitleBlocks` + SHEET |

### Build / Run (npm, aus `web/`)
| Script | Wirkung |
|---|---|
| `npm run build-wasm` | `wasm-pack build --target web --dev` + `normalize-wasm.cjs` → `web/wasm/{wasm.js,wasm_bg.wasm,wasm.d.ts}` |
| `npm run build` | `build-wasm` + `stencil build` + `copy-index.cjs` → Output `web/www/` (inkl. `index.html`, `build/`, `wasm/`, `webspice-symbols/`) |
| `npm run dev` | `build` + `serve -l 8080 www` |
| `npm run serve` | statischer Server `serve -l 8080 www` |
| `npm run test` | `stencil test --spec` (Unit-Tests für `render.ts`) |

Rebuild-Skript (außerhalb von npm): `web/build.ps1` (Windows) bzw.
`web/build.sh` (bash) — idempotent, in einem Aufruf `npm run build`
(build-wasm + stencil build + index-Kopie) **und** erzeugen danach
`web/www/.nojekyll`; Details: Sektion „Deployment".

### Deployment (GitHub Pages, Task 12)

Veröffentlichung des Build-Outputs `web/www/` über GitHub Pages
("Deploy from branch") — ohne CI (CI-Punkt liegt im Backlog, s. Tasklog).

- **Branch `gh-pages`**: Inhalt = Inhalte von `web/www/` im **Branch-Root**
  (via `git subtree push`). Begründung: Pages' Branch-Deployment liefert nur
  `/ (root)` oder `/docs` eines Branches — `web/www/` liegt dagegen auf
  `main` unter `web/www/` und `/docs` ist belegt (Format-Spezifikationen),
  daher eigener Branch mit dem Build-Output als Root.
- **Publish-Reihenfolge**: (1) Rebuild via `web/build.ps1`/`web/build.sh`,
  (2) Commit von `web/www/` auf `main`, (3) User pusht:
  `git subtree push --prefix web/www origin gh-pages`
  (der Agent committet/pusht nicht, AGENTS.md §Git and files),
  (4) einmalig in GitHub: Settings → Pages → Source = "Deploy from
  branch", Branch `gh-pages`, Ordner `/ (root)`.
- **`.nojekyll`**: muss **vom Build-Skript** in `web/www/.nojekyll` erzeugt
  werden (nicht von Hand), weil `stencil build` `www/` neu schreibt
  (sonst würde Jekyll `.wasm`/Punkte-Dateien filtern).
- **Relative-Path-Audit**: die Site läuft unter `/<user>/<repo>/`,
  daher sind nur **relative** Ressourcen-Referenzen erlaubt:
  - `web/index.html`: `src="./build/ascviewer.esm.js"` (kein `/build/…`),
  - Stencil-ESM-Bundle: relative `./p-…`-Imports, `resourcesUrl` aus
    `import.meta.url` (Bundle intern, keine Änderung nötig),
  - WASM (`web/src/wasm.ts`) und `webspice-symbols/*.asy`
    (`schematic-viewer.tsx`): relativ zu `document.baseURI` aufgelöst,
  - CSS: keine absoluten `url()`/Fonts.
  Guard-Test: `web/src/deploy.spec.ts` (assertet, dass `index.html`
  keine absoluten `src="/…"`, `href="/…"`, `url(/…)`-Referenzen trägt).
- **Git**: `web/.gitignore` ignoriert `www/` nicht mehr (Build-Output wird
  committed; erzeugt nur über das Build-Skript); Root-`.gitignore`: `target/`.
- **Acceptance**: öffentliche URL `https://<user>.github.io/<repo>/` lädt
  ohne Konsolen-Fehler; `wasm/wasm_bg.wasm` und `webspice-symbols/*.asy`
  werden nicht mit 404 ausgeliefert (Pfad-Check); eine `.asc`-Fixture lädt
  und rendert; Screenshot der Live-Seite → "Evidence (GUI-Abnahme)".

### Evidence (GUI-Abnahme)
- `web/screenshots/mvp-success.png` — gültige Fixture: Wires/Flags/Texte/
  Graphics + Symbol-Boxen sichtbar
- `web/screenshots/mvp-error.png` — Fehler-Fixture: Error-Panel mit
  Zeilennummer, letztes Rendering bleibt sichtbar
- `web/screenshots/symbols-asy.png` — Task 09: `jtk3.asc` mit Kurzbibliothek:
  R/C/D/… zeichnen echte `.asy`-Geometrie (Statuszeile „Symbole: 188/192");
  fehlende Entries (`fuse`, `L7809`) bleiben Platzhalter-Boxen
- `web/screenshots/symbols-asy-fixed.png` — Bugfix 2026-11-13:
  gespiegelte Symbole (R11/R13 `M0`, C10 `M180`) sitzen exakt in den
  Wire-Lücken (vorher zu weit rechts/unten)
- `web/screenshots/ltspice-look.png` — Task 10: `jtk3.asc` +
  Kurz-Bibliothek im LTspice-Look gegen den Referenz-Screenshot:
  TREBLE/BASS/VOLUME/MID (R180/M180) **upright** lesbar, blaue
  Wires/Netlabels/Erd-Symbol, Knotenpunkte (Grid-Punkte),
  Marker an unverdrahteten Pins, Tahoma bold in
  `render-config.json`-Größen
- `web/screenshots/ltspice-fidelity.png` — Task 11: `jtk3.asc` Tonstack
  LTspice-treu — `InstName`/`Value`/`SpiceLine` an Window-Slot-Position
  (Slot 0/3/39) mit Window-Justification, nur Junction-Punkte (keine
  48-LE-Mittelpunkte), US-Zickzack-Resistor, runde Unverdrahtet-Pin-Marker,
  `SpiceLine`+`.asy`-TEXT non-bold vs. bold `InstName`/`Value`
- `web/screenshots/ltspice-fidelity.png` (Abgleich-Detaille, jtk3.asc):
  `VTop`-Label mittig über Anker / `VBottom`-Label mittig unter Anker
  (Task 14-Baseline), (R24: `R24`/`100k` beide x-zentriert), `res` als
  europäisches Rechteck (Standard-Bibliothek = `sym/`-Originale; `current`
  ergänzt), Junction-Punkte nur bei > 2 Verbindungsenden
  (2-Draht-Ecke ohne Punkt), V3b Value (`WINDOW … Invisible`) fehlt,
  keine Pin-Punkte
- `web/screenshots/ltspice-fidelity2.png` — Task 14 (LTspice-Abgleich 2,
  jtk3.asc D1/D2/C30): schwarze InstName/Value + `.asc`-TEXT
  (Default `fontSize` 10, size 4 → 13.3), C30/D1 right-aligned
  (rechter Textrand am Anker), D2-Label horizontal, Junction-Quadrate
  `6`×`6`, kein Text-Auswahl beim Pan/Zoom (`user-select: none`)
- `web/screenshots/ltspice-fidelity2-r24.png` — Task 14, R24-Detail
  (jtk3.asc): `R24`/`100k` horizontal und schwarz am rotierten (R90)
  EU-Rechteck-Resistor (Gegenrotation der Window-Labels), `.asc`-TEXT
  (`TREBLE`/`BASS`) schwarz vs. Netlabel (`CHAN-C`) in netlabelColor
- `web/screenshots/gh-pages-local-subpath.png` — Task 12, lokaler Subpfad-Check
  (`web/` als Server-Root, App unter `/www/`, wie auf GitHub Pages):
  `jtk3.asc` gerendert (Status `… symbols: 192 … Symbole: 188/192`,
  619 Wires/42 Flags), alle 27 Ressourcen (Bundle, `wasm_bg.wasm`,
  18× `.asy`, `.nojekyll`) mit 200, keine Konsolenfehler
- `web/screenshots/gh-pages-live.png` — Task 12, Live-Abnahme
  (https://simonbethke.github.io/webspice/): `jtk3.asc` gerendert
  (Status `… symbols: 192 … Symbole: 188/192`), alle 26 Requests mit 200
  (Bundle, `wasm.js`, 18× `.asy`), keine Konsolenfehler

### Grenzen
- `.asy`: keine `SYMKEY`-/Parameter-Bindung, Window-Slots nur {0,3,39},
  keine `DRAW`-Varianten, keine `.lib`/`.inc`
- keine Multi-Page (SHEET > 1), keine Hover-/Klick-Interaktion, kein Netlist-Export

## Required test cases
**Ab Task 01**
- `parse_bytes(b"")` → leeres `Schematic`
- Inhalt nur aus Whitespace → leeres `Schematic`
- Unbekanntes Token in jeder Zeile → roh erhalten (`Symbol.extras` im
  Symbol-Scope, sonst `Schematic.extras`), kein Fehler
- CRLF ≡ LF (gleicher Parser-Ergebnis)
- Tab-getrennte Argumente ≡ Leerzeichen-getrennte
- Nicht-UTF-8-Eingabe (z. B. cp1252 `0xB5` = µ in `10µ`) → Windows-1252-Decodierung,
  Parse läuft ohne Fehler weiter (verlustfrei; undefinierte cp1252-Bytes
  0x81/0x8D/0x8F/0x90/0x9D → C1-Kontrollzeichen)
- `Display` aller zeilenbezogenen Error-Varianten enthält `line <n>`

**Ab Task 02**
- `VERSION 4` → `version = Some("4")`; `VERSION 4.1` → `Some("4.1")`
- `SHEET 1 880 680` → `sheet = Some(Sheet { page: 1, width: 880, height: 680 })`
- `WIRE 160 96 224 96` → `wires` enthält den korrekten Wert
- `FLAG 160 96 0` → `flags` enthält `name == "0"` (Ground)
- `BUSTAP 160 96` → Punkt in `bustaps`
- Mixed-Datei mit allen 5 Tokens → alle Felder gefüllt
- Fehlerfälle (je mit Zeilennummer): `VERSION a b` → `UnexpectedToken`;
  `SHEET 1 880` → `MissingField`; `SHEET 0 …` / negative width/height →
  `InvalidNumber`; `WIRE 1 2 abc 4` → `InvalidNumber`;
  `FLAG 1 2 net extra` → `UnexpectedToken`

**Ab Task 03**
- SYMBOL + 3× SYMATTR (`InstName`, `Value`, `SpiceLine`) → Attribut-Map korrekt
- Zwei aufeinanderfolgende Symbole → Attribute mischen sich nicht
- SYMBOL ohne Attribute → leere Map
- SYMBOL als letzte Zeile → EOF-Flush funktioniert
- `SYMATTR` ohne aktives SYMBOL → `OrphanedSymAttr` (mit Zeilennummer)
- SYMATTR-Wert mit Leerzeichen → verbatim (innere bleiben)
- SYMBOL mit 3 bzw. 5 Nachfolgern → `MissingField` / `UnexpectedToken`
- Orientation `R45` → `InvalidToken`; alle 8 gültigen Tokens parsen
- Duplizierte SYMATTR-Keys → letzter Wert gewinnt

**Ab Task 04**
- `!`-Direktive (z. B. `!.tran 10m`) → `is_command == true`, Text verbatim
- `;`-Kommentar → `is_command == false`
- Freitext ohne Präfix → `is_command == false`
- Mehrfach-Leerzeichen im Inhalt bleiben erhalten
- Alle drei Alignment-Tokens (`Left`/`Center`/`Right`)
- `size` bzw. `content` fehlt → `MissingField` mit Zeilennummer
- Ungültiges `align` (`top`) → `InvalidToken`
- TEXT-Zeile nach offenem SYMBOL schließt das Symbol

**Ab Task 05**
- Alle 4 Primitives × alle 4 Styles (16 Fälle, Felder exakt prüfen)
- ARC mit 7 Nachfolgern → `MissingField` mit Zeilennummer;
  `LINE`/`RECTANGLE`/`CIRCLE`/`ARC` mit exakt einem numerischen Schluss-Flag →
  `flag` gesetzt; ≥ 2 zusätzliche Tokens → `UnexpectedToken`
- Ungültiger Style (`DottedD`) → `InvalidToken`
- Negative Koordinaten werden akzeptiert (nur Syntax-Check)
- Graphic-Zeile nach offenem SYMBOL schließt das Symbol

**Ab Task 06**
- UTF-8-BOM am Datei-Anfang → wird ignoriert (Parsing läuft normal weiter)
- Datei mit nur BOM → leeres Schematic
- Umlaute/Sonderzeichen in TEXT- und FLAG-Inhalten bleiben erhalten
- Leere Datei → leeres Schematic; sehr lange Zeile (~100k Zeichen) parst ohne Hängen
- Integration: jede Fixture aus `tests/fixtures/*.asc` parst via `parse_file`
  mit den erwarteten Strukturen
- Fehler-Fixtures → exakte Fehler-Variante mit der erwarteten Zeilennummer

**Bugfix 2026-11-12 (`WINDOW` / unbekannte Tokens)**
- `SYMBOL` + `WINDOW` + `SYMATTR` (Reihenfolge echter Dateien) → die
  `SYMATTR`-Zeile gehört zum Symbol; `Symbol.windows` enthält die
  `WINDOW`-Einträge
- Unbekanntes Token (z. B. `IOPIN`, `DATAFLAG`) zwischen `SYMBOL` und
  `SYMATTR` → kein Fehler, kein `OrphanedSymAttr`, Zeile roh in
  `Symbol.extras`
- Unbekannte Zeile ohne offenes Symbol → roh in `Schematic.extras`
- Deformierte `WINDOW`-Zeile (falsche Argumentenzahl) → roh in
  `Symbol.extras`, kein Fehler

**Ab Task 09 (`.asy`-Symbolen)**
- Synthetische `.asy`: `Version`, `SYMBOLTYPE CELL`/`BLOCK`, `SYMATTR`,
  `WINDOW` (`<index> <x> <y> <just> <n>`, z. B. `0 31 40 Left 2`),
  `LINE`/`RECTANGLE`/`CIRCLE`/`ARC`
  (Styles `Normal` + `Wide` → `LineStyle::Bold`), `TEXT` (Inhalt mit
  Leerzeichen), `PIN` (mit und ohne Name, auch `+`/`-`), `PINATTR`-Kette
  (letzter gewinnt) → `AsySymbol` mit exakt erwarteten Feldern
- Fehlende `Version`-Zeile (erste nicht-leere Zeile) → `MissingVersion`,
  leere Datei → `line 1`
- `PINATTR` ohne vorhergehendes `PIN` (z. B. Zeile 5) →
  `OrphanedPinAttr { line: 5 }`
- `SYMBOLTYPE GURU` → `InvalidToken`; `WINDOW 0 1 2 Left` (4 Argumente)
  → `MissingField`; `WINDOW 0 1 2 Left 2 3` (6) → `UnexpectedToken`;
  `WINDOW 0 x 2 Left 1` → `InvalidNumber`; `PIN` mit 3 Argumenten →
  `MissingField`; `LINE Normal 1 2` → `MissingField`;
  `Dotted`/`Dashed`-Style → `InvalidToken`
- UTF-8-BOM am Datei-Anfang → ignoriert; unbekannte Zeilen → still ignoriert
- Case-insensitive Keywords: `line normal …`, `symattr …` parsen
- Integration: `tests/asy-fixtures/*.asy` (mind. `res`, `cap`, `diode`,
  `npn`, `pot`, `pot_log`, `opamp`, `EuropeanPolcap`, `EuropeanResistor`)
  via `parse_asy_bytes`; `asy_to_json` → erwartetes JSON
- Web-Renderer: Orientation-Transform je Token — `R0` → kein Zusatztransform,
  `R90` → `rotate(90)`, `R180` → `rotate(180)`, `R270` → `rotate(270)`,
  `M0` → `scale(-1 1)`, `M90` → `rotate(-90) scale(-1 1)`,
  `M180` → `rotate(-180) scale(-1 1)`, `M270` → `rotate(-270) scale(-1 1)`;
  gleicher Transform bei aufgelöstem `.asy`-Symbol und Fallback-Box
- GUI: `web/screenshots/symbols-asy.png` — echte `.asy`-Geometrie für
  R/C/OP-Amp + Fallback-Box für fehlendes Symbol

**Ab Task 10 (Web-Renderer, `render.spec.ts`)**
- Upright-Text: Text-Wrapper (Value/InstName, `.asy`-Text, Fallback-Label)
  bei allen 8 Orientierungen — Gegenrotations-Strings: `R0` → keins,
  `R90` → `rotate(-90)`, `R180` → `rotate(-180)`, `R270` →
  `rotate(-270)`, `M0` → `scale(-1 1)`, `M90` → `matrix(0 1 1 0 0 0)`,
  `M180` → `scale(1 -1)`, `M270` → `matrix(0 -1 -1 0 0 0)`; vertikale
  `V`-Texte kombinieren Gegenrotation + `rotate(-90)`;
  **Akzeptanz:** `M180`-Poti (MID) → Text-Transforms enthalten weder
  `180` noch `scale(-1 …)` (spiegelungsfrei)
- Config: Ad-hoc-`RenderConfig` ändert die korrespondierenden
  SVG-Attribute — `fontFamily`, `fontSize`, `fontBold` (font-weight),
  `wireColor`/`penThickness`, `symbolColor`, `textColor`,
  `netlabelColor`, `pinColor`, `gridPointSize` (Junction-Quadrat-
  Kantenlänge), `gridPointColor`, `unconnectedPinColor`, Toggles
- `.asc`-`TEXT`-Größenregel: size 4 → klar größer, size 2 → klar
  kleiner als Bauteil-Label (Regel `fontSize·size/3`, 1 Dezimale)
- Erd-Symbol bei `FLAG 0` (Stange + 3 abnehmende Balken, `symbolColor`,
  viewBox +18 LE unten); andere `FLAG`-Namen → Netlabel-Text
  (`netlabelColor`)
- Grid-Punkte (Task 11): nur Junctions (> 2 Verbindungsenden am
  exakten Punkt; Wire-Enden + aufgelöste Pin-World-Enden);
  bei `showGridPoints: false` → keine
- Unverdrahteter Pin → Kreis-Marker (Radius 2, `unconnectedPinColor`),
  verdrahteter Pin (Wire-Endpunkt ≤ 1 LE) → kein Marker;
  bei `markUnconnectedPins: false` → keine
- Blatt-Rahmen (SHEET) bei `showTitleBlocks: true` (stroke
  `wireColor`, hinten), **nicht** in der viewBox
- Default-Config = JSON-Import (`render-config.json`): Tahoma/10/
  bold; Labels `textColor` `#000000`, rest `#0000C0`/`#C00000`;
  `gridPointSize` 6; Toggles an
- GUI: `web/screenshots/ltspice-look.png` — `jtk3.asc` +
  Kurz-Bibliothek: TREBLE/BASS/VOLUME/MID upright, blaue
  Wires/Netlabels/Erd-Symbol, Knotenpunkte, bold Tahoma

**Ab Task 11 (Web-Renderer, `render.spec.ts`)**
- Window-Slots: `InstName` (Slot 0) / `Value` (Slot 3) / `SpiceLine`
  (Slot 39) in der Window-Position mit Window-Justification (Task-14-
  edge-Modell) — `Left` → `end`+`central`, `VLeft` → `start`+`central`,
  `Center`/`VCenter` → `middle`+`central`, `Right` → `start`+`central`,
  `VRight` → `end`+`central`,
  `Top`/`VTop` → `middle`+`text-after-edge` (Text über Anker),
  `Bottom`/`VBottom` → `middle`+`text-before-edge` (Text unter
  Anker); **keine Rotation** (im Gegensatz zu `.asy`-`TEXT`);
  Instanz-`WINDOW` schlägt `.asy`-`WINDOW`
- `.asy`-Window-Fallback: Instanz ohne eigene `WINDOW`
  (z. B. `R23`/`R13`, `res`) → `Value`/`SpiceLine` aus dem `.asy`
  (`WINDOW` + `SYMATTR`)
- Skip-Regel: `Value`/`SpiceLine` ohne auffindbares Window (Instanz
  **und** `.asy`) → nicht gerendert; `InstName` ohne Window →
  Default-Position (0,0), `start`; aufgelöstes Window `Invisible`
  → Slot-Label wird nicht gerendert (kein Fallback-Render)
- `.asc`-`TEXT` Präfix-Strip: Zeilen mit führendem `;` bzw. `!` rendern
  ohne das erste Zeichen; Zeilen ohne Präfix unverändert; `is_command`
  im AST bleibt bei `!`-Zeilen `true`
- Grid-Punkte (Junction-Matrix): 3-Draht-T → Punkt; Kreuzung
  (4 Enden) → Punkt; Pin+Pin+Draht (3 Enden) → Punkt; 2-Draht-Ecke
  (2 Enden) → **kein** Punkt; Pin+1 Draht (2 Enden) → **kein**
  Punkt; freies Draht-Ende (1) → **kein** Punkt; Flag/BUSTAP/Text
  zählen nicht als Verbindungsenden; `showGridPoints: false` →
  keine Punkte
- Gewicht: `SpiceLine` und `.asy`-`TEXT` → `font-weight: normal`;
  `InstName`/`Value`/`.asc`-`TEXT` bold (Config `fontBold`)
- Unverdrahtete Pins: Marker = `<circle>` (Radius 2), kein `<rect>`;
  keine Pin-`<circle>`-Dots in der Symbol-Darstellung (Pin-Endpunkte
  nutzen nur die Junction- und Unverdrahtet-Regel)
- GUI: `web/screenshots/ltspice-fidelity.png` — `jtk3.asc` Tonstack
  vs. LTspice-Referenz: Label+SpiceLine an Window-Position, Junction-Punkte,
  Zickzack-Resistor, runde Pin-Marker, bold/dünn-Kontrast
- GUI: `web/screenshots/ltspice-fidelity.png` (Abgleich-Detaille) —
  (`jtk3.asc`): VTop-Label mittig über, VBottom-Label mittig unter
  dem Anker (R24: `R24`/`100k` beide x-zentriert), `res` als
  europäisches Rechteck (Standard-Bibliothek = `sym/`-Originale),
  Junction-Matrix wie oben (T/Kreuz → Punkt, 2-Drähte → kein Punkt),
  V3b Value (`Invisible`) fehlt, keine Pin-Punkte

**Ab Task 14 (LTspice-Optik-Abgleich 2, `render.spec.ts`)**
- Window-Justification (edge-Modell): Plain-`Left` → `text-anchor: end`
  (rechte Textkante am Anker, Text wächst nach links), Plain-`Right` →
  `start` (linke Textkante am Anker, Text wächst nach rechts),
  `VLeft` → `start`, `VRight` → `end`, `Center`/`VCenter` → `middle`
  (alle Horizontal-Tokens `dominant-baseline: central`);
  `Top`/`VTop` → `middle` + `text-after-edge` (Text über Anker),
  `Bottom`/`VBottom` → `middle` + `text-before-edge` (Text unter Anker)
  — 4 Abgleich-Fälle: C30 `M0`/`Left`→`end`; D1 `R180` name `Left`→
  `end` + value `VRight`→`end`; D2 `R270` `VTop`/`VBottom`→`middle`+
  Baselines; R24 `R90` `VBottom`/`VTop`→`middle`+Baselines
- 90°/270°-gedrehte Bauteile: InstName/Value bleiben **horizontal**
  (nur Position/Justierung/Größe/Farbe, keine Textrichtung; kein
  zusätzlicher Text-Rotations-Transform)
- Farben: `textColor` `#000000` (schwarz) für InstName/Value/`.asc`-
  TEXT/`.asy`-TEXT; Netlabels/BUSTAP/Flags/Pins/Wires/Symbols bleiben
  `netlabelColor`/`symbolColor`/`wireColor`/`pinColor` `#0000C0`-Familie
- Schriftgröße: Default `fontSize` `10`; `.asc`-`TEXT`-Regel
  `fontSize·size/3` skaliert mit (size 4 → 13.3, size 2 → 6.7);
  Netlabels/InstName/Value/`.asy`-TEXT → `10`
- Grid-Punkt-Größe: neue config `gridPointSize` Default `6` (vorher
  hardcoded `3`); Junction-Quadrate `6`×`6`, Position `p.x-3`/`p.y-3`
- Viewer: `user-select: none` (keine Textauswahl beim Pan/Zoom;
  `schematic-viewer.css`)
- GUI: `web/screenshots/ltspice-fidelity2.png` — `jtk3.asc` (D1/D2/
  C30/`INPUT1`), Vergleich mit `image.png`-Referenz: schwarze
  InstName/Value, kleinere `.asc`-TEXT-Größen (13.3 statt 37.3),
  C30/D1 right-aligned (rechter Textrand am Anker), Junction-Quadrate
  `6`×`6`, kein Text-Auswahl beim Pan

**Ab Task 12 (Deployment)**
- `web/index.html` enthält keine absoluten Ressourcen-Referenzen
  (`src="/…"`, `href="/…"`, `url(/…)`), da die Site unter
  `/<user>/<repo>/` ausgeliefert wird (Guard-Test: `web/src/deploy.spec.ts`)
- Rebuild-Skript erzeugt `www/.nojekyll` (idempotent; Build-Skript erzeugt
  es nach jedem `npm run build`, s. Sektion „Deployment")

## Changelog
- 2026-11-14 — GitHub-Pages-Deployment (Task 12): **Relative-Path-Audit**
  (`web/index.html`: `src="/build/…"` → `src="./build/ascviewer.esm.js"`;
  Bundle/WASM/Symbol-Fetch waren bereits relativ); **Rebuild-Skripte**
  `web/build.ps1` / `web/build.sh` (idempotent, ein Aufruf:
  `npm run build` + erzeugt `www/.nojekyll`); **Guard-Test**
  `web/src/deploy.spec.ts` (keine absoluten Ressourcen-Referenzen in
  `index.html`); `web/.gitignore` ignoriert `www/` nicht mehr
  (Build-Output wird committed) + `www/`-Kommentar, Root-`.gitignore`
  (`target/`); lokaler Subpfad-Check: `jtk3.asc` rendert, alle 27
  Ressourcen 200, keine Konsolenfehler (siehe „Evidence").
  Publish (User): `git subtree push --prefix web/www origin gh-pages` +
  Pages-Settings, s. Sektion „Deployment“; **Live-Abnahme:** Seite live unter
  https://simonbethke.github.io/webspice/ — `jtk3.asc` gerendert
  (Symbole 188/192), alle 26 Requests mit 200, keine Konsolenfehler
  (siehe „Evidence“: `gh-pages-live.png`).
- 2026-11-13 — LTspice-Optik-Abgleich 2 (jtk3.asc, `image.png`-Referenz;
  Task 14): (1) **Window-Justification edge-Modell**: Plain-`Left` →
  `end` (rechte Textkante am Anker, Text wächst nach links), Plain-`Right`
  → `start` (linke Textkante am Anker); `VLeft`/`VRight`/`Center`/
  `VCenter` unverändert; `Top`/`VTop` und `Bottom`/`VBottom` → `middle`
  + Baselines umgedreht (`Top` → `text-after-edge` = Text über Anker,
  `Bottom` → `text-before-edge` = Text unter Anker); dadurch C30/D1
  right-aligned wie LTspice, D2/R24 korrekt positioniert; 90°/270°-
  Bauteile rendern Labels horizontal (nur Position/Justierung/Größe/
  Farbe, keine Textrichtung). (2) **Farben**: `textColor` `#000080` →
  `#000000` (InstName/Value/`.asc`-TEXT/`.asy`-TEXT); Netlabels/BUS/
  Pins/Wires/Symbols bleiben `#0000C0`-Familie. (3) **Schriftgröße**:
  `fontSize` `28` → `10` (Default; `.asc`-`TEXT`-Regel `fontSize·size/3`
  skaliert mit: 37.3→13.3, 18.7→6.7, 93.3→33.3). (4) **Grid-Punkt-
  Größe**: neu config-gesteuertes `gridPointSize` (Default `6`, vorher
  hardcoded `3`). (5) **Viewer**: `schematic-viewer.css` `user-select:
  none` (keine Textauswahl beim Pan/Zoom). Code:
  `web/src/render/render-config.json`, `web/src/render/render.ts`
  (`RenderConfig`-Interface + `gridPointSize`, `windowAnchor`,
  `windowBaseline`, `gridPointEls`), `web/src/render/render.spec.ts`
  (Assertions + neue Task-14-Teste),
  `web/src/components/core/schematic-viewer/schematic-viewer.css`;
  `web/screenshots/ltspice-fidelity2.png` neu; Rust unverändert.
- 2026-11-13 — Doku-Aufräumen: Spezifikationen nach `docs/spec/`, Sektion
  „Projektstruktur“ neu, „Required test cases“ ans Ende (vor Changelog),
  Stand auf Task 11, Tippfehler/Einrückungen; `AGENTS.md` neu
  (webspice-spezifisch, Template-Reste entfernt). Code unverändert.
- 2026-11-13 — LTspice-Abgleich (jtk3.asc, 3 Abweichungen):
  (1) **Window-Justification edge-Modell**: das Token benennt die
  Textkante am Ankerpunkt — `Top`/`VTop` → `middle` +
  `text-before-edge` (Text unter Anker), `Bottom`/`VBottom` →
  `middle` + `text-after-edge` (Text über Anker), `Left`/`VLeft` →
  `start`+`central`, `Center`/`VCenter` → `middle`+`central`,
  `Right`/`VRight` → `end`+`central`; dadurch sind
  `Top`/`Bottom`-Labels jetzt horizontal zentriert (vorher
  `start`); `Invisible` → Slot-Label wird nicht gerendert (kein
  Fallback), realer Fall jtk3 V3b (`WINDOW 3 31 47 Invisible 2`).
  (2) **Pins**: Pin-Dots (`<circle r=2>` in `pinColor`) entfernt,
  `pinColor` jetzt nur für `BUSTAP`; Pins fließen in Junction- und
  Unverdrahtet-Regel ein.
  (3) **Junction-Matrix**: Grid-Punkte nur bei > 2 Verbindungsenden
  am exakten Punkt (Wire-Enden + aufgelöste Pin-World-Enden);
  2-Draht-Ecke und Pin+Draht (2 Enden) → kein Punkt (Verhalten
  ändert sich gegenüber „≥ 2 Enden" + Nicht-Kollinearität);
  Flag/BUSTAP/Text werden nicht gezählt.
  (4) **Standard-Bibliothek**: `web/webspice-symbols/res.asy` →
  `sym/res.asy` (europäisches Rechteck, byte-identisch zu
  `EuropeanResistor.asy`; Local-Frame-BBox identisch),
  `current.asy` aus `sym/` ergänzt; `sym/` hat kein `gnd.asy` →
  FLAG `0` bleibt code-gezeichnet.
  Code: `web/src/render/render.ts`, `web/src/render/render.spec.ts`,
  `web/webspice-symbols/res.asy`,
  `web/webspice-symbols/current.asy`; Rust unverändert.
- 2026-11-13 — Task 11: LTspice-Referenz-Korrekturen im Web-Renderer.
  (1) **Labels via WINDOW-Slots**: `InstName` (Slot 0) / `Value` (Slot 3) /
  `SpiceLine` (Slot 39) werden an der Instanz-`WINDOW`-Position mit
  Window-Justification gerendert (horizontal `Left`/`Center`/`Right` →
  `start`/`middle`/`end`; vertikal `VTop`/`VBottom` →
  `text-before-edge`/`text-after-edge`, sonst `central`; keine Rotation);
  Fallback auf `.asy`-`WINDOW` + `SYMATTR Value`/`SpiceLine`;
  `Value`/`SpiceLine` ohne auffindbares Window (Instanz **und** `.asy`)
  → nicht gerendert; `InstName` ohne Window → Default (0,0), `start`.
  (2) **`.asc`-`TEXT`-Prefix-Strip**: führendes `;`/`!` wird bei der
  Anzeige gestrippt (AST/`is_command` unverändert).
  (3) **Gewichtung**: `SpiceLine` und `.asy`-`TEXT` → `font-weight: normal`;
  `InstName`/`Value`/`.asc`-`TEXT` bleiben bold (Config `fontBold`).
  (4) **Grid-Punkte → Junctions**: `showGridPoints` markiert nur noch echte
  Junctions (≥ 2 Wire-Endpunkte am selben Punkt); die 48-LE-Innenpunkt-Regel
  ist entfernt; `GRID_STEP`-Inner-Loop entfernt.
  (5) **Unverdrahtete Pins** → gefüllte Kreise (Radius 2) statt 3×3-Quadrate.
  (6) **`res.asy`** → US-Zickzack (Zickzack-Linien statt Rechteck).
  Code: `web/src/types/ast.ts` (`Symbol.windows: WindowSpec[]`),
  `web/src/render/render.ts`, `web/src/render/render.spec.ts`,
  `web/webspice-symbols/res.asy`; Rust unverändert.
- 2026-11-13 — Bugfix (Task 10): `COUNTER_ROT` für `M90`/`M270` war eine
  4-Argument-Matrix (`matrix(0 1 1 0)`), die Browser als ungültige
  `transform` verwerfen; jetzt vollständige 6-Argument-Form
  `matrix(0 1 1 0 0 0)` bzw. `matrix(0 -1 -1 0 0 0)` (Mathematik unverändert,
  `render.spec.ts` mitgeprüft)
- 2026-11-13 — Task 10: LTspice-Optik im Web-Renderer, komplett
  config-gesteuert (`web/src/render/render-config.json`, TS-Interface
  `RenderConfig` in `render.ts`; `renderSchematic(..., config?)` mit
  JSON-Default, `resolveJsonModule`): (1) **Upright-Text** in
  gedrehten/gespiegelten Symbolen — `.asy`-`TEXT`, `Value`, `InstName`,
  Fallback-Label werden pro Orientation gegenrotiert (Grafik+Pins
  bleiben im Orientation-Transform), vertikale Texte kombinieren mit
  `rotate(-90)`; (2) Theme/Typografie aus der Config (Tahoma bold,
  blau `#0000C0`, Strich `penThickness`, Bold = 2×); (3) `FLAG 0` →
  LTspice-Erd-Symbol, andere Flags → **Netlabels**; `BUSTAP`/Pin-Punkte
  in Config-Farben; `.asc`-`TEXT`-Größenregel `fontSize·size/3`;
  (4) Toggles: `showGridPoints` (Knoten + 48-LE-Interiores),
  `markUnconnectedPins` (Pin-Endpunkte ohne Wire ≤ 1 LE),
  `showTitleBlocks` (SHEET-Rahmen, außerhalb der viewBox);
  Tests in `render.spec.ts` aktualisiert/erweitert, neue
  Evidence `web/screenshots/ltspice-look.png`
- 2026-11-13 — Bugfix: `.asc`-Orientation `M0`/`M90`/`M180`/`M270` werden
  jetzt gespiegelt statt auf 0° gemappt (davor fielen gespiegelte Symbole,
  z. B. R11/R13 `M0` und C10 `M180` in `jtk3.asc`, zu weit rechts/unten auf);
  Mapping empirisch aus Pin↔Wire-Endpunkten abgeleitet, Docs („Geometrie",
  AST→SVG-Tabelle, Grenzen), Tests und Evidence
  `web/screenshots/symbols-asy-fixed.png` aktualisiert
- 2026-11-12 — Task 09: Symbol-Rendering via `.asy` (neue Sektion
  „Symbol-Rendering (.asy)"): `parse_asy_bytes`/`asy_to_json` + WASM
  `parse_asy`, AST `AsySymbol`/`AsyText`/`AsyPin`, `Wide` → `LineStyle::Bold`,
  neue Fehler `MissingVersion`/`OrphanedPinAttr`, Web: Kurzbibliothek
  `web/webspice-symbols/` + `.asy`-Replace via Filepicker/Drop, Tests in
  `tests/asy-fixtures/`, GUI-Evidence `web/screenshots/symbols-asy.png`
- 2026-11-12 — Bugfix: optionales numerisches Schluss-Flag auf
  `LINE`/`RECTANGLE`/`CIRCLE`/`ARC`-Zeilen wird toleriert und als
  `flag: Option<i32>` im `GraphicPrimitive` abgelegt (echte LTspice-Dateien
  schreiben z. B. `RECTANGLE Normal … 2`); `jtk3.asc` parst damit komplett
- 2026-11-12 — Bugfix: `WINDOW`-Zeilen und unbekannte Tokens (`IOPIN`,
  `DATAFLAG`, …) schließen nicht mehr das offene Symbol; `WINDOW` → typisiert in
  `Symbol.windows`, unbekannte Zeilen → roh in `Symbol.extras` /
  `Schematic.extras` (Roundtrip); echte Dateien mit
  `SYMBOL`/`WINDOW`/`SYMATTR`-Reihenfolge (z. B. `jtk3.asc`) parsen jetzt
- 2026-11-12 — Bugfix: Nicht-UTF-8-Eingabe wird verlustfrei als
  Windows-1252 (cp1252) dekodiert (z. B. `0xB5` = µ in `SYMATTR Value 10µ`);
  `Error::InvalidUtf8` entfällt
- 2026-11-12 — Task 08: Web-Frontend MVP (Stencil v4 + TS in `web/`):
  `schematic-viewer` (Filepicker/Drop, Statuszeile, Error-Panel, Pan/Zoom),
  pure `render.ts` (AST → SVG-DOM), normalisiertes WASM-Paket `web/wasm/`,
  GUI-Evidence in `web/screenshots/`
- 2026-11-11 — Task 07: WASM-Export `parse(bytes)→JSON`, `serde`/`serde_json`-Deps,
  `schematic_to_json`-Host-API, JSON- & Fehler-Vertrag für das Web-Frontend
- 2026-11-11 — Task 06: Robustheit (UTF-8-BOM wird ignoriert),
  Integrationstests mit Fixtures, finale Abnahme
- 2026-11-11 — Task 05: grafische Primitives `LINE`/`RECTANGLE`/`CIRCLE`/`ARC`
  (mit `LineStyle::from_token`, Argument-Validierung)
- 2026-11-11 — Task 04: Parsing von `TEXT` (verbatim-Inhalt inkl. `!`/`;`-Präfix,
  `is_command`, `align`-/`size`-Validierung)
- 2026-11-11 — Task 03: SYMBOL/SYMATTR-Statemachine (Flush, `OrphanedSymAttr`,
  verbatim-Werte, Duplikat-Regel "letzter gewinnt")
- 2026-11-11 — Task 02: Parsing von `VERSION`, `SHEET`, `WIRE`, `FLAG`, `BUSTAP`
  (inkl. Argument- und Wert-Validierung)
- 2026-11-11 — initial: Skeleton (API, Fehlermodell, AST, Parser-Schleife);
  Token-Logik folgt in Task 02–05
