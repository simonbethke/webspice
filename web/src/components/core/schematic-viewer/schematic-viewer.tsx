import { Component, Element, h } from '@stencil/core';
import { Box, contentBox, renderSchematic } from '../../../render/render';
import { AsySymbol, Schematic } from '../../../types/ast';
import { parseAsc, parseAsy } from '../../../wasm';

const MIN_K = 0.1;
const MAX_K = 50;

/**
 * Statische Kurzbibliothek `web/webspice-symbols/` (committed, docs
 * „Web-Frontend“): Basenamen (ohne Erweiterung) der beim Start geladenen
 * `.asy`-Dateien; SYMBOL-Namen aus `.asc` werden case-insensitiv dagegen
 * aufgelöst (unauflösbar → Platzhalter-Box).
 */
const SYMBOL_LIBRARY: readonly string[] = [
  'cap',
  'diode',
  'EuropeanPolcap',
  'EuropeanResistor',
  'ind',
  'ind2',
  'LED',
  'nmos',
  'npn',
  'pjf',
  'pot',
  'pot_log',
  'res',
  'spdt',
  'spst',
  'triode',
  'voltage',
  'zener',
];

interface ViewBox {
  x: number;
  y: number;
  w: number;
  h: number;
}

interface Point {
  x: number;
  y: number;
}

/**
 * Schematic-Viewer: `.asc`-Datei laden (Picker/Drag&Drop), parsen via WASM,
 * als SVG rendern. `.asy`-Symbol-Dateien (Picker/Drag&Drop) ersetzen den
 * Kurzbibliotheks-Eintrag mit demselben Basenamen (höhere Priorität) und
 * lösen damit echte Symbol-Geometrien statt Platzhalter-Boxen (docs
 * „Web-Frontend“). Pan (Drag) + Zoom (Wheel, `+`/`−`/`1:1`/`Fit`) über
 * einen `transform: translate scale` auf `<g class="viewport">`.
 *
 * Hinweis: Status/Error-Domänen werden imperativ aktualisiert, nicht über
 * `@State`. Ein Re-Render würde den imperativ in das `<svg>` gerenderten
 * Inhalt entfernen (Stencil patcht nur deklarative VNodes); so bleibt das
 * zuletzt erfolgreiche Rendering bei Fehlern sichtbar.
 */
@Component({
  tag: 'schematic-viewer',
  styleUrl: 'schematic-viewer.css',
  shadow: true,
})
export class SchematicViewer {
  @Element() el!: HTMLElement;

  private svgEl: SVGSVGElement | null = null;
  private viewportEl: SVGElement | null = null;
  private box: Box | null = null;
  private k = 1;
  private tx = 0;
  private ty = 0;
  private dragging = false;
  private lastClient: Point | null = null;
  private statusEl: HTMLElement | null = null;
  private errorEl: HTMLElement | null = null;
  /** Kurzbibliothek `webspice-symbols/`: lowercased Basename → `.asy`-Symbol. */
  private symbols = new Map<string, AsySymbol>();
  /** Zuletzt erfolgreich geladenes Schematic (Re-Render bei `.asy`-Austausch). */
  private ast: Schematic | null = null;

  componentDidLoad(): void {
    const root = this.el.shadowRoot;
    if (root === null) return;
    const svg = root.querySelector('svg');
    this.svgEl = svg !== null ? (svg as SVGSVGElement) : null;
    this.statusEl = root.querySelector('.status');
    this.errorEl = root.querySelector('.error-panel');
    void this.loadSymbolLibrary();
  }

  private setStatus(text: string): void {
    if (this.statusEl !== null) {
      this.statusEl.textContent = text;
      this.statusEl.hidden = false;
    }
  }

  private setError(message: string | null): void {
    if (this.errorEl !== null) {
      this.errorEl.hidden = message === null;
      this.errorEl.textContent = message !== null ? message : '';
    }
  }

  private parseViewBox(): ViewBox {
    const raw = this.svgEl !== null ? this.svgEl.getAttribute('viewBox') : null;
    if (raw !== null) {
      const p = raw.split(/[\s,]+/).filter((t) => t !== '').map(Number);
      if (p.length === 4 && p.every((n) => Number.isFinite(n))) {
        return { x: p[0], y: p[1], w: p[2], h: p[3] };
      }
    }
    return { x: 0, y: 0, w: 100, h: 100 };
  }

  /** Client-Pixel → viewBox-Koordinaten (vor dem Viewport-Transform). */
  private clientToViewBox(p: Point): Point {
    const svg = this.svgEl;
    const ctm = svg !== null ? svg.getScreenCTM() : null;
    if (ctm === null) return p;
    const inv = ctm.inverse();
    return {
      x: inv.a * p.x + inv.c * p.y + inv.e,
      y: inv.b * p.x + inv.d * p.y + inv.f,
    };
  }

  private applyTransform(): void {
    if (this.viewportEl !== null) {
      this.viewportEl.setAttribute('transform', `translate(${this.tx} ${this.ty}) scale(${this.k})`);
    }
  }

  /** Zoom um einen Punkt in viewBox-Koordinaten (k wird in 0.1…50 geklemmt). */
  private zoomAt(c: Point, factor: number): void {
    const k0 = this.k;
    const k1 = Math.max(MIN_K, Math.min(MAX_K, k0 * factor));
    if (k1 === k0) return;
    const ratio = k1 / k0;
    this.tx = c.x - ratio * (c.x - this.tx);
    this.ty = c.y - ratio * (c.y - this.ty);
    this.k = k1;
    this.applyTransform();
  }

  private zoomCenter(factor: number): void {
    const vb = this.parseViewBox();
    this.zoomAt({ x: vb.x + vb.w / 2, y: vb.y + vb.h / 2 }, factor);
  }

  /** Ansicht auf Faktor `k` zurücksetzen, Inhalt zentriert. */
  private resetView(k: number): void {
    const vb = this.parseViewBox();
    const vbCx = vb.x + vb.w / 2;
    const vbCy = vb.y + vb.h / 2;
    const b = this.box;
    const ccx = b !== null ? (b.minX + b.maxX) / 2 : 0;
    const ccy = b !== null ? (b.minY + b.maxY) / 2 : 0;
    this.k = Math.max(MIN_K, Math.min(MAX_K, k));
    this.tx = vbCx - this.k * ccx;
    this.ty = vbCy - this.k * ccy;
    this.applyTransform();
  }

  /** Alles Sichtbare in den viewBox-Rand (Inhalt zentriert). */
  private fitView(): void {
    const vb = this.parseViewBox();
    const b = this.box;
    let k = 1;
    if (b !== null) {
      const cw = Math.max(1, b.maxX - b.minX);
      const ch = Math.max(1, b.maxY - b.minY);
      k = Math.min(vb.w / cw, vb.h / ch);
    }
    this.resetView(k);
  }

  private async loadFile(file: File): Promise<void> {
    if (file.name.toLowerCase().endsWith('.asy')) {
      await this.loadAsyFile(file);
      return;
    }
    try {
      const t0 = Date.now();
      const bytes = new Uint8Array(await file.arrayBuffer());
      const json = await parseAsc(bytes);
      const ms = Date.now() - t0;
      const ast: Schematic = JSON.parse(json);

      this.setError(null);
      if (this.svgEl === null) return;
      this.ast = ast;
      this.viewportEl = renderSchematic(ast, this.svgEl, this.symbols);
      this.box = contentBox(ast);
      this.resetView(1);
      const r = this.resolution();
      this.setStatus(
        `${file.name} · ${bytes.byteLength} bytes · Parse: ${ms} ms · ` +
          `wires: ${ast.wires.length}, flags: ${ast.flags.length}, ` +
          `bustaps: ${ast.bustaps.length}, symbols: ${ast.symbols.length}, ` +
          `texts: ${ast.texts.length}, graphics: ${ast.graphics.length} · ` +
          `Symbole: ${r.resolved}/${r.total}`
      );
    } catch (err) {
      // Fehler-Vertrag (docs): geworfener String enthält `line N: …`.
      // Das zuletzt erfolgreiche Rendering bleibt sichtbar.
      this.setError(typeof err === 'string' ? err : String(err));
    }
  }

  /**
   * `.asy`-Datei laden (Picker/Drop): ersetzt den Kurzbibliotheks-Eintrag
   * mit demselben Basenamen (höhere Priorität als die Kurzbibliothek,
   * docs „Web-Frontend“). Ist ein Schematic geladen, Re-Render mit der
   * aktualisierten Bibliothek; aktueller Pan/Zoom-Bestand bleibt erhalten.
   */
  private async loadAsyFile(file: File): Promise<void> {
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const json = await parseAsy(bytes);
      const sym = JSON.parse(json) as AsySymbol;
      const key = file.name.replace(/\.asy$/i, '').toLowerCase();
      this.symbols.set(key, sym);
      this.setError(null);
      this.refreshSchematic();
      const r = this.resolution();
      this.setStatus(
        `${file.name} als Symbol geladen · ` +
          (this.ast !== null
            ? `Schematic re-rendered · Symbole: ${r.resolved}/${r.total}`
            : 'noch kein Schematic geladen')
      );
    } catch (err) {
      // Fehler-Vertrag (docs): geworfener String enthält `line N: …`.
      this.setError(typeof err === 'string' ? err : String(err));
    }
  }

  /**
   * Re-Render des zuletzt geladenen Schematics mit der aktuellen
   * Kurzbibliothek. Der neue Viewport-`<g>` startet unvertransformiert,
   * daher `applyTransform()`, um k/tx/ty neu anzuwenden.
   */
  private refreshSchematic(): void {
    const ast = this.ast;
    if (ast === null || this.svgEl === null) return;
    this.viewportEl = renderSchematic(ast, this.svgEl, this.symbols);
    this.applyTransform();
  }

  /** Auflösung im letzten Schematic: `<aufgelöst>/<gesamt>` (docs Statuszeile). */
  private resolution(): { resolved: number; total: number } {
    const ast = this.ast;
    if (ast === null) return { resolved: 0, total: 0 };
    let resolved = 0;
    for (const s of ast.symbols) {
      if (this.symbols.has(s.name.toLowerCase())) {
        resolved += 1;
      }
    }
    return { resolved, total: ast.symbols.length };
  }

  /**
   * Statische Kurzbibliothek `webspice-symbols/` via WASM `parse_asy()` ins
   * `symbols`-Register laden (docs „Web-Frontend“). Fehlende oder defekte
   * Dateien werden still übersprungen — betroffene SYMBOLs bleiben
   * Platzhalter-Boxen; hier ist kein Fehlerzustand des Viewers.
   */
  private async loadSymbolLibrary(): Promise<void> {
    for (const name of SYMBOL_LIBRARY) {
      try {
        const sym = await this.fetchAsySymbol(name);
        if (sym !== null) this.symbols.set(name.toLowerCase(), sym);
      } catch {
        // Datei fehlt (404) oder Parse-Fehler → Platzhalter-Box bleibt Fallback.
      }
    }
    this.refreshSchematic();
  }

  /**
   * `webspice-symbols/<name>.asy` laden und parsen → `AsySymbol`;
   * `null` bei HTTP-Fehlern (Datei fehlt, z. B. in manchen Dev-Setups).
   */
  private async fetchAsySymbol(name: string): Promise<AsySymbol | null> {
    const base = typeof document !== 'undefined' ? document.baseURI : 'http://localhost:8080/';
    const res = await fetch(new URL(`webspice-symbols/${name}.asy`, base).href);
    if (!res.ok) return null;
    const bytes = new Uint8Array(await res.arrayBuffer());
    const json = await parseAsy(bytes);
    return JSON.parse(json) as AsySymbol;
  }

  private onFileChange = (ev: Event): void => {
    const input = ev.target as HTMLInputElement;
    const file =
      input.files !== null && input.files.length > 0 ? input.files[0] : null;
    input.value = '';
    if (file !== null) void this.loadFile(file);
  };

  private onDragOver = (ev: Event): void => {
    ev.preventDefault();
  };

  private onDrop = (ev: Event): void => {
    ev.preventDefault();
    const de = ev as DragEvent;
    const files =
      de.dataTransfer !== null && de.dataTransfer !== undefined
        ? de.dataTransfer.files
        : null;
    const file = files !== null && files.length > 0 ? files[0] : null;
    if (file !== null) void this.loadFile(file);
  };

  private onPointerDown = (ev: PointerEvent): void => {
    if (ev.button !== 0 || this.svgEl === null) return;
    this.dragging = true;
    this.lastClient = { x: ev.clientX, y: ev.clientY };
    try {
      this.svgEl.setPointerCapture(ev.pointerId);
    } catch (_err) {
      // setPointerCapture nicht verfügbar → Drag läuft trotzdem über Pointermove.
    }
  };

  private onPointerMove = (ev: PointerEvent): void => {
    if (!this.dragging || this.lastClient === null || this.svgEl === null) {
      return;
    }
    const p0 = this.clientToViewBox(this.lastClient);
    const p1 = this.clientToViewBox({ x: ev.clientX, y: ev.clientY });
    this.tx += p1.x - p0.x;
    this.ty += p1.y - p0.y;
    this.lastClient = { x: ev.clientX, y: ev.clientY };
    this.applyTransform();
  };

  private onPointerUp = (ev: PointerEvent): void => {
    this.dragging = false;
    this.lastClient = null;
    if (this.svgEl !== null && this.svgEl.hasPointerCapture(ev.pointerId)) {
      this.svgEl.releasePointerCapture(ev.pointerId);
    }
  };

  private onWheel = (ev: WheelEvent): void => {
    if (this.svgEl === null) return;
    ev.preventDefault();
    const factor = ev.deltaY < 0 ? 1.25 : 1 / 1.25;
    this.zoomAt(this.clientToViewBox({ x: ev.clientX, y: ev.clientY }), factor);
  };

  render() {
    return (
      <div class="viewer">
        <div class="toolbar">
          <label class="file-picker">
            .asc laden:
            <input type="file" accept=".asc" onChange={this.onFileChange} />
          </label>
          <label class="file-picker">
            .asy laden:
            <input type="file" accept=".asy" onChange={this.onFileChange} />
          </label>
          <span class="hint">oder hierher ziehen (.asc / .asy)</span>
          <div class="zoom">
            <button type="button" title="Hineinzoomen" onClick={() => this.zoomCenter(1.25)}>
              +
            </button>
            <button type="button" title="Herauszoomen" onClick={() => this.zoomCenter(1 / 1.25)}>
              −
            </button>
            <button type="button" title="1:1" onClick={() => this.resetView(1)}>
              1:1
            </button>
            <button type="button" title="Fit" onClick={() => this.fitView()}>
              Fit
            </button>
          </div>
        </div>
        <div class="status" hidden>
          Bereit — .asc-Schematic oder .asy-Symbol per Picker oder hierher ziehen.
        </div>
        <div class="error-panel" hidden></div>
        <div class="canvas" onDragOver={this.onDragOver} onDrop={this.onDrop}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 100 100"
            preserveAspectRatio="xMidYMid meet"
            onPointerDown={this.onPointerDown}
            onPointerMove={this.onPointerMove}
            onPointerUp={this.onPointerUp}
            onWheel={this.onWheel}
          ></svg>
        </div>
      </div>
    );
  }
}
