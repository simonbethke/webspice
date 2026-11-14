#!/usr/bin/env bash
# Rebuild-Skript (idempotent, ein Aufruf) — docs Sektion „Deployment" (Task 12);
# bash-Äquivalent von web/build.ps1:
#   1) npm run build  =  build-wasm (WASM -> web/wasm/, normalisiert)
#                        + stencil build (-> web/www/) + index-Kopie
#   2) web/www/.nojekyll erzeugen (muss vom Build-Skript kommen,
#      nicht von Hand — Stencil schreibt www/ bei jedem Rebuild neu).
set -euo pipefail
cd "$(dirname "$0")"
npm run build
: > www/.nojekyll
echo "build: www/.nojekyll geschrieben"
