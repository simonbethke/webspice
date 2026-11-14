# Tasklog

Source of truth für Reihenfolge und Status der Tasks in `tasks/`.
Ein Task gilt als fertig, wenn: Code komplett, `docs/documentation.md` synchron,
`cargo clippy … -D warnings`, `cargo test` und `cargo fmt --all` sauber
(bei Web-Änderungen zusätzlich `npm run test` in `web/`).
Dann: Datum + Uhrzeit hier unter der Task-Headline in "Completed" notieren
und die Task-Datei löschen. Danach beginnt der nächste Task.

Reihenfolge: strikt aufsteigend nach Nummer.

## Pending
- 13 — GitHub Action Build+Deploy bei Push (ultimativ) (`tasks/13-github-action.md`)

## Completed

### 12 — GitHub Pages, schnellster funktionierender Weg
2026-11-14 17:58

### 14 — LTspice-Optik-Abgleich 2 (jtk3.asc, LTspice-Referenz)
2026-11-14 15:01

### 11 — Web-Rendering: LTspice-Referenz-Korrekturen (jtk3-Tonestack)
2026-11-13 15:37

### 10 — Rendering: LTspice-Optik, config-gesteuert
2026-11-13 12:35

### 09 — Symbol-Rendering aus .asy (erste Erweiterung nach MVP)
2026-11-13 06:33

### 08 — Web-Frontend MVP, Stencil+TS, lokal
2026-11-12 18:29

### 07 — WASM-Export & JSON-API
2026-11-11 22:42

### 06 — Hardening & Integration
2026-11-11 18:29

### 05 — Graphics-Primitives
2026-11-11 18:17

### 04 — TEXT
2026-11-11 17:58

### 03 — SYMBOL & SYMATTR (State Machine)
2026-11-11 17:53

### 02 — Meta & Topologie
2026-11-11 17:47

### 01 — Skeleton & Modell
2026-11-11 17:30
