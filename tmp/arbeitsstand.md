# Arbeitsstand – webspice LTspice-Fidelity-Fix (Stand: 2026-11-13, Ende Session)

## Auftrag (vom Nutzer, nicht yet in Task-Queue)
Restliche visuelle Abweichungen des Web-Renderers gegenüber Original-LTspice (Referenz: `jtk3.asc`):
1. **Labels** (InstName/Value) pro WINDOW-Slot auf den Anchor zentrieren; `VTop`/`VBottom`-Baseline
   korrekt (aktuell links-ankert und vertauscht). `Invisible`-Token: Label nicht rendern.
2. **Symbole**: Default-Bibliothek = Originale aus `sym\` (res, cap, ind, diode, voltage, current).
   GND = Balken-Flag (FLAG 0), da kein gnd-.asy im Korpus existiert.
3. **Knotenpunkte (Junction Dots)**: nur bei ≥ 3 zusammenfallenden Verbindungsenden
   (Draht-Enden + aufgelöste Symbol-Pins, Weltkoordinaten, Toleranz ~1 LE).
   Keine Pin-Punkte, keine 2-Draht-Ecken.

## Verifizierte Fakten (diese Session, Hash-/Byte-Nachweis)
- **Pfad**: Projekt = `D:\rust\webspice` 
- **Aktive Symbol-Library**: `web\webspice-symbols\` (18 .asy) — bestätigt per
  `stencil.config.ts:16` (`src: '../webspice-symbols'`) und Fetch-URL im schematic-viewer.
- **Stale Duplikat**: `web\weaspice-symbols\` (a statt b, 18 Dateien, identisch außer res.asy)
  → nicht geladen, Deletion-Kandidat (Nutzer abnicken).
- **MD5-Vergleich** (korpus `sym\` vs. aktive Lib):
  - cap.asy, ind.asy, diode.asy, voltage.asy: **byte-identisch** mit Korpus ✓ (schon Originale).
  - res.asy aktiv = **525 B selbstgebautes Zigzag** ≠ `sym\res.asy` (429 B, EU-Rechteck,
    "European style") ≠ `sym\res2.asy` (472 B, US-Zigzag, Pins (16,0)/(16,64),
    `WINDOW 0 36 16 Left` / `WINDOW 3 36 56 Left`).
    ⇒ Falls das Original-LTspice-Zigzag der Standard-US-Resistor ist → Inhalt von
    `sym\res2.asy` übernehmen (Pin-Pitch 64 statt 80 LE ändert das Layout → GUI-Vergleich!).
- **current.asy**: im Korpus (471 B), in **beiden** Web-Dirs **fehlend**; jtk3 nutzt
  `current`-Quellen nicht (21 Symbolnamen, davon `fuse` + `L7809` = dokumentierte Fallback-Boxen).
- **Kein gnd/ground .asy** irgendwo in `sym\` (rekursiv geprüft) → GND bleibt `flagEl`
  (GROUND_H=18, Balken-Halbw. 9/6/3); Proportion nur per Screenshot-Vergleich prüfen.
- **jtk3.asc WINDOW-Tokens** (gebrauchte Vokabel!): Left 105, VBottom 65, VTop 65, Center 8,
  VLeft 5, Bottom 3, VRight 2, Invisible 1, VCenter 1. **Kein** Top/Right. FLAG-Zeilen: 42.
- **Label-Beleg (R24)**: `jtk3.asc` L1338-1341:
  `SYMBOL res 288 -848 R90` / `WINDOW 0 5 56 VBottom 2` / `WINDOW 3 27 56 VTop 2` / `InstName R24`.
  R90-Mapping (px,py)→(x−py, y+px) ⇒ Anchor W0 (232,−843) = Körper-Oberrand, W3 (232,−821) =
  Körper-Unterrand. Original: "R24" **über** dem Körper, "100k" **unter**, beide zentriert
  ⇒ **VBottom → text-before-edge (Text über Anchor)**, **VTop → text-after-edge (Text unter
  Anchor)**, `text-anchor: middle`.
- **Invisible-Bezug**: `WINDOW 3 31 47 Invisible 2` (triode, ~L920) → Value-Label weglassen.
- **Code-Stand** (`web\src\render\render.ts`): `windowAnchor` ~L317 (nichts-VToken → left = Bug A),
  `windowBaseline` ~L325 (VTOP/VBOTTOM vertauscht = Bug B), `flagEl` ~L351, `asySymbolEl` ~L519
  (PIN→`<circle r=2>` entfernen), `gridPointEls` ~L639 (2-Drähte-Ecke durch ≥3-Enden-Regel ersetzen),
  `unconnectedPinEls` ~L712 (behalten).
- **Tasks**: 01–11 Completed (tasklog), 12/13 Pending (GitHub Pages/Action).
  Dieser Fix ist **noch keine Task** — bei Start entsprechend in `tasks/` anlegen? (abklopfen).
- **GUI-Evidence**: `web\screenshots\` (6 PNGs, neueste `ltspice-fidelity.png` 13.11. 15:34).
  Vorgeschriebener Mechanismus: Docs-Sektionen **"Evidence (GUI-Abnahme)"** + **"Debug
  rendering"** (bevor implementiert wird, diese 2 Sektionen neu lesen; AGENTS.md nennt
  `cargo run --release -- --debug-render <state.json>` → `screenshot.jpg`).
  Original-LTspice-Bilder sind **nicht im Repo** (Nutzer-Screenshots) → für endgültigen
  Vergleich Original anfordern.

## Geplane neue Token-Semantik (11 Tokens, erst in Docs, dann Code)
| Token      | anchor   | baseline          |
|------------|----------|-------------------|
| Left       | start    | Mitte             |
| Center     | middle   | Mitte             |
| Right      | end      | Mitte             |
| Top*(nicht belegt) | middle | text-after-edge |
| Bottom     | middle   | text-before-edge  |
| VLeft      | start    | text-before-edge  |
| VCenter    | middle   | text-before-edge  |
| VRight     | end      | text-before-edge  |
| VTop       | middle   | text-after-edge (belegt R24) |
| VBottom    | middle   | text-before-edge (belegt R24) |
| Invisible  | —        | nicht rendern     |

## Umgebungs-Fakten / Tool-Fallen
- Read-Tool schlägt sporadisch fehl; Shell `Get-Content -Encoding UTF8` zuverlässig
  (Standard-Codepage verdreht Umlaute!).
- PowerShell-Casts `(int)[char]$_` fallen in diesem Wrapper durch → statt `UTF8.GetBytes` nutzen.
- node v22.18.0, npm 11.12.0, cargo 1.96.1. Web: `npm run test` (stencil --spec), `npm run build`.
- Keine neuen Dependencies, kein rayon. Kein Commit ohne Auftrag.
- Keine echten Multi-GB-Testdaten; nur synthetische Fixtures.

## Reihenfolge nächste Session
1. Docs-Sektionen lesen: "Auflösung & Rendering (Web)" (Token-/Junction-/Pin-Passagen),
   "AST → SVG-Mapping", "Required test cases", "Evidence (GUI-Abnahme)", "Debug rendering".
2. **Docs zuerst** aktualisieren (Token-Tabelle, Junction-Regel, Pin-Punkte raus,
   Symbol-Tabelle + changelog-Zeile oben).
3. Code: `render.ts` (Anchor/Baseline/Invisible, gridPointEls, PIN-Circle removal),
   Library: `res.asy` → Korpus-Original (welches? s. unten), `current.asy` ergänzen.
4. `render.spec.ts`: Zentrierung, Junction T/Cross-Fall, keine Pin-Punkte, Invisible-Skip.
5. `npm run test` + `npm run build`; GUI-Screenshot nach Docs-Mechanismus; Vergleich mit
   `web\screenshots\ltspice-fidelity.png` + Original-Anforderung.
6. Offene Fragen an Nutzer (max. je 1):
   (a) R-Zeichnung im Original: Standard-US-Zigzag (→ `sym\res2.asy`), EU-Rechteck
   (→ `sym\res.asy`) oder das jetzige Custom-Zigzag beibehalten?