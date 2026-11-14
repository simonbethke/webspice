const fs = require('fs');
const t = fs.readFileSync('docs/documentation.md', 'utf8');
const lines = t.split(/\r?\n/);
lines.forEach((l, i) => {
  const c = (l.match(/`/g) || []).length;
  if (
    /wasm-pack|out-dir|\.d\.ts|WASM|Web-Export/.test(l) ||
    c % 2 !== 0
  ) {
    console.log((i + 1) + ' |' + c + '| ' + JSON.stringify(l.slice(0, 150)));
  }
});
