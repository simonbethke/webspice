/**
 * Deployment-Guard (docs Sektion „Deployment", Task 12): die Site wird
 * über GitHub Pages unter dem Subpath `/<user>/<repo>/` ausgeliefert,
 * darf daher in `index.html` keine absoluten Ressourcen-Referenzen
 * (`src="/…"`, `href="/…"`, `url(/…)`) enthalten — nur relative Pfade.
 */
import * as fs from 'fs';
import * as path from 'path';

// __dirname = web/src/  =>  index.html liegt um eine Ebene höher (web/index.html)
const INDEX_HTML = path.join(__dirname, '..', 'index.html');

/** Fängt absolut referenzierte Ressourcen in HTML/CSS-Attributen. */
function absoluteResourceRefs(html: string): string[] {
  const patterns = [
    /src\s*=\s*["']\/[^"'>\s]*["']/g,
    /href\s*=\s*["']\/[^"'>\s]*["']/g,
    /url\(\s*["']?\/[^"')]*["']?\s*\)/g,
  ];
  const refs: string[] = [];
  for (const re of patterns) {
    for (const m of html.match(re) ?? []) {
      refs.push(m);
    }
  }
  return refs;
}

describe('deploy/index.html relative paths (GitHub Pages)', () => {
  test('keine absoluten src="/…", href="/…" oder url(/…)-Referenzen', () => {
    const html = fs.readFileSync(INDEX_HTML, 'utf8');
    expect(absoluteResourceRefs(html)).toEqual([]);
  });
});
