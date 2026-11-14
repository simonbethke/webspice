# 12 — GitHub Pages (schnellster funktionierender Weg)

Ziel: Seite öffentlich erreichbar, ohne CI — Build-Output committed.

## Entscheidung (fix, nicht mehr offen)
GitHub Pages "Deploy from branch" liefert nur den Root oder `/docs` eines Branches
aus, keinen beliebigen Unterpfad. `web/www` auf `main` scheidet daher aus, `/docs`
ebenfalls (belegt durch `docs/documentation.md`, `docs/spec/`).
Gewählt: **Branch `gh-pages`**, Inhalt = der Inhalt von `web/www/` im Branch-Root.
- `web/www/` wird auf `main` committed (Build-Output, nur über das Build-Skript erzeugt).
- Veröffentlichen: `git subtree push --prefix web/www origin gh-pages`
  (führt der User aus; der Agent committet/pusht nicht, siehe AGENTS.md §Git and files).
- GitHub-Einstellung (User, einmalig): Settings → Pages → Source = "Deploy from branch",
  Branch `gh-pages`, Ordner `/ (root)`.

## Umfang
- **Relative-Path-Audit**: Stencil-Loader, WASM (`import.meta.url`-basierte
  `.wasm`-Auflösung aus wasm-pack), CSS/Fonts, `webspice-symbols/`-Fetches —
  keine absoluten `/…`-Pfade (Pages liefert unter `/<repo>/`-Subpfad!)
- `.nojekyll` im Root von `web/www/`: muss **vom Build-Skript** erzeugt werden
  (nicht von Hand), weil `stencil build` `www/` neu schreiben kann
- Rebuild-Skript `web/build.ps1` (+ bash-Äquivalent): idempotent, `build-wasm` +
  `build` + `.nojekyll` in einem Aufruf
- Acceptance: öffentliche URL lädt ohne Konsole-Fehler; eine `.asc`-Fixture
  lädt und rendert; Screenshot der Live-Seite committen (Evidence);
  `.wasm`-Datei und `webspice-symbols/*.asy` kommen nicht mit 404 (Pfad-Check)
- Docs: Abschnitt **"Deployment"** (Branch-Wahl samt Begründung, Pfad, Skript,
  Rebuild- und Publish-Reihenfolge, Pages-Einstellung); Changelog.
- `README.md`: nicht anfassen, aber im Summary vermerken, wenn obsolet
  (AGENTS.md §Communication).

## Nicht im Scope
CI/CD (Task 13), Custom-Domain, Analytics.
