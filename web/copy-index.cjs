// Kopiert index.html nach www/ (Stencil www-Ziel kopiert es in dieser
// Projekt-Konfiguration nicht automatisch; wir machen es explizit,
// deterministisch und Cross-Plattform).
const fs = require('fs');
const path = require('path');
const from = path.join(__dirname, 'index.html');
const to = path.join(__dirname, 'www', 'index.html');
fs.copyFileSync(from, to);
console.log('copy-index: ' + to);
