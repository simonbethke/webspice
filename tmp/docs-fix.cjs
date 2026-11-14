const fs = require('fs');
const f = 'docs/documentation.md';
const t = fs.readFileSync(f, 'utf8');
let out = t.replace(
  /`wasm-pack build --target web --dev --out-dir web\/[^\n`]*`\n/,
  '`npm run build-wasm` in `web/` (wasm-pack + `normalize-wasm.cjs`,\n' +
    '  Output `web/wasm/`, siehe Abschnitt „Web-Frontend (MVP)")\n'
);
out = out.replace(/`web\/[A-Za-z0-9._-]*\.d\.ts`/, '`web/wasm/wasm.d.ts`');
if (out === t) {
  console.log('NO MATCH — nothing replaced');
  process.exit(1);
}
fs.writeFileSync(f, out);
console.log('replaced OK; len', t.length, '->', out.length);
