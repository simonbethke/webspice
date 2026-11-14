// Normalisiert das wasm-pack-Output auf das stabile, committed-Paket
// web/wasm/ mit den Namen:
//   wasm.js  wasm_bg.wasm  wasm.d.ts  wasm_bg.wasm.d.ts  package.json
// und korrigiert die Selbst-Referenzen im Paket:
//   - Binär-URL in wasm.js         -> 'wasm_bg.wasm'
//   - @ts-self-types in wasm.js    -> './wasm.d.ts'
//   - package.json main/types/files -> stabile Namen
// Bemerkung: der Import-Key "./<crate>_bg.js" in wasm.js bleibt UNVERÄNDERT —
// er ist der Modulname im Wasm-Binary (Import-Sektion), kein Dateipfad.
//
// Idempotent: läuft sowohl auf frischem wasm-pack-Output (neben web/
// erzeugtes Verzeichnis) als auch auf dem bereits normalisierten web/wasm.
'use strict';

const fs = require('fs');
const path = require('path');

const root = __dirname; // web/
const WANTED = 'wasm';

// 1) Frisches wasm-pack-Output-Verzeichnis suchen und nach web/wasm verschieben.
const isWasmPackDir = (p) =>
  fs.existsSync(path.join(p, 'package.json')) &&
  fs.readdirSync(p).some((f) => f.endsWith('.wasm') && !f.endsWith('.d.ts'));

const dirs = fs
  .readdirSync(root, { withFileTypes: true })
  .filter((d) => d.isDirectory())
  .map((d) => d.name);

const srcDir = dirs.find(
  (n) => n !== WANTED && n !== 'node_modules' && isWasmPackDir(path.join(root, n))
);

if (srcDir) {
  const from = path.join(root, srcDir);
  const to = path.join(root, WANTED);
  if (fs.existsSync(to)) fs.rmSync(to, { recursive: true, force: true });
  fs.renameSync(from, to);
  console.log(`normalize: moved ${srcDir}/ -> ${WANTED}/`);
}

const wasmDir = path.join(root, WANTED);
if (!fs.existsSync(path.join(wasmDir, 'package.json'))) {
  console.log('normalize: no wasm output dir found, nothing to do');
  process.exit(0);
}

// 2) Dateinamen normalisieren (Reihenfolge: _bg.wasm.d.ts vor .d.ts).
const renameIf = (pred, target) => {
  const f = fs.readdirSync(wasmDir).find(pred);
  if (f && f !== target) {
    fs.renameSync(path.join(wasmDir, f), path.join(wasmDir, target));
    console.log(`normalize: renamed ${f} -> ${target}`);
  }
};
renameIf((f) => f.endsWith('_bg.wasm.d.ts'), 'wasm_bg.wasm.d.ts');
renameIf((f) => f.endsWith('.d.ts'), 'wasm.d.ts');
renameIf((f) => f.endsWith('.wasm'), 'wasm_bg.wasm');
renameIf((f) => f.endsWith('.js'), 'wasm.js');

// 3) Selbst-Referenzen in wasm.js korrigieren (Token-extraktion via Regex,
//    d. h. unabhängig vom konkreten Crate-Namen; idempotent).
const jsPath = path.join(wasmDir, 'wasm.js');
let js = fs.readFileSync(jsPath, 'utf8');
const before = js;
js = js.replace(/new URL\(\s*["'][^"']*_bg\.wasm["']/, "new URL('wasm_bg.wasm'");
js = js.replace(/@ts-self-types=["']\.[^"']*?\.d\.ts["']/, '@ts-self-types="./wasm.d.ts"');
if (js !== before) {
  fs.writeFileSync(jsPath, js);
  console.log('normalize: patched self-references in wasm.js');
}

// 4) package.json auf stabile Namen setzen (idempotent).
const pkgPath = path.join(wasmDir, 'package.json');
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
let pkgChanged = false;
const setIf = (key, val) => {
  if (JSON.stringify(pkg[key]) !== JSON.stringify(val)) {
    pkg[key] = val;
    pkgChanged = true;
  }
};
setIf('type', 'module');
setIf('main', 'wasm.js');
setIf('types', 'wasm.d.ts');
setIf('files', ['wasm.js', 'wasm_bg.wasm', 'wasm.d.ts', 'wasm_bg.wasm.d.ts']);
if (pkgChanged) {
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log('normalize: patched package.json');
}

// 5) Das wasm-pack-eigene .gitignore würde das Paket verbergen —
//    wir committen das Paket vollständig.
const giPath = path.join(wasmDir, '.gitignore');
const giText = '# Paket ist ein Build-Artefakt — vollständig committen.\n';
if (!fs.existsSync(giPath) || fs.readFileSync(giPath, 'utf8') !== giText) {
  fs.writeFileSync(giPath, giText);
}

console.log('normalize: done');
