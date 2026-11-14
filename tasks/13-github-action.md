# 13 — GitHub Action: Build & Deploy bei Push (ultimativ)

Ziel: Push auf `main` → automatisches Build + Deploy auf Pages,
nachvollziehbar und cache-bar.

## Workflow (`.github/workflows/deploy.yml`)
- Trigger: `push branches: [main]` (+ `workflow_dispatch`)
- Build-Job:
  - `actions/setup-node@v4` (Node 20 LTS), `dtolnay/rust-toolchain@stable`
  - `rustup target add wasm32-unknown-unknown`
  - wasm-pack-Install: `cargo binstall wasm-pack` (Version pinnen!)
  - Caches: `cargo` (`target/`, Registry), `npm` (`node_modules/`)
  - `web/build.ps1` bzw. bash-Äquivalent (Task 12)
  - Build-Profil: `--dev` (MVP); `--release` + `optimization-level="sz"`
    nur bei konkretem Load-Time-Problem (dann messen, nicht spekulieren)
- Deploy-Job: GitHub Pages Deploy (`actions/deploy-pages` /
  `actions/upload-pages-artifact`) mit `www/`-Inhalt
- Fehlfälle: jeder Failing-Run muss lesbaren Output liefern
  (wasm-pack-Version echo, `stencil build`-Level normal)

## Acceptance
- Push → Deploy E2E < 5 min; Live-Seite identisch zum Task 12-Zustand;
  1× Cache-Hit bei zweitem Lauf (Log prüfen); Screenshot/Evidence wie Task 12
- Docs: Abschnitt "Deployment" um CI-Erweiterung ergänzen; Changelog.

## Nicht im Scope
Multi-Build (Matrix), Preview-Deploys pro PR, Monitoring-Alerts.
