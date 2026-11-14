# Rebuild-Skript (idempotent, ein Aufruf) — docs Sektion „Deployment" (Task 12):
#   1) npm run build  =  build-wasm (WASM -> web/wasm/, normalisiert)
#                        + stencil build (-> web/www/) + index-Kopie
#   2) web/www/.nojekyll erzeugen (muss vom Build-Skript kommen,
#      nicht von Hand — Stencil schreibt www/ bei jedem Rebuild neu).
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
  npm run build
  if ($LASTEXITCODE -ne 0) {
    throw "npm run build failed (exit $LASTEXITCODE)"
  }
  Set-Content -Path (Join-Path $PSScriptRoot 'www\.nojekyll') -Value '' -NoNewline
  Write-Host 'build: www/.nojekyll geschrieben'
} finally {
  Pop-Location
}
