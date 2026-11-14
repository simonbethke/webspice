/**
 * Web-SVG-Renderer: wandelt den Schematic-AST (types/ast.ts) plus optional
 * aufgelöste .asy-Symbole in ein SVG auf dem Host-Element um.
 *
 * Reine Logik: kein I/O, kein Browser-API außer DOM-Generierung,
 * deterministisch.
 *
 * Theme/Typos/Toggles: `RenderConfig` (Default = `render-config.json`,
 * `defaultConfig`). Siehe docs/documentation.md "Auflösung & Rendering (Web)".
 */

import type {
  AsySymbol,
  AsyText,
  GraphicPrimitive,
  LineStyle,
  Orientation,
  Schematic,
  Symbol,
  TextDirective,
  Wire,
  WindowSpec,
} from '../types/ast';

import renderConfigJson from './render-config.json';

const SVG_NS = 'http://www.w3.org/2000/svg';

/** FLAG-Name, der das LTspice-Erd-Symbol bedeutet (Task 10). */
function flagIsGround(name: string): boolean {
  return name.trim() === '0';
}

/** Erd-Symbol: Stange unterhalb der Flag-Position (LE). */
const GROUND_H = 18;
/** Unverdrahtete-Pins: Draht-Endpunkt-Distanz-Schwelle (LE). */
const PIN_TOLERANCE = 1;

/**
 * Render-Konfiguration (Task 10): Theme, Typos und Display-Toggles.
 * Werte come aus `render-config.json` (Default-Config `defaultConfig`).
 */
export interface RenderConfig {
  fontFamily: string;
  fontSize: number;
  fontBold: boolean;
  penThickness: number;
  wireColor: string;
  symbolColor: string;
  textColor: string;
  netlabelColor: string;
  pinColor: string;
  gridPointSize: number;
  gridPointColor: string;
  unconnectedPinColor: string;
  showTitleBlocks: boolean;
  markUnconnectedPins: boolean;
  showGridPoints: boolean;
}

/** Default-Config: eingebettete `render-config.json`. */
export const defaultConfig: RenderConfig = renderConfigJson as RenderConfig;

// ---------------------------------------------------------------------------
// Content-Box / viewBox
// ---------------------------------------------------------------------------

// Platzhalter-Symbol-Box (Task 08): 32×14 um den Symbol-Anchor (±16 / ±7).
const PLACEHOLDER_HALF_W = 16;
const PLACEHOLDER_HALF_H = 7;

/** Axis-aligned Bounding-Box (Leinheiten). */
export interface Box {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

/** Bounding-Box aller Content-Elemente (ohne SHEET-Rahmen), `null` wenn leer. */
export function contentBox(ast: Schematic): Box | null {
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  let has = false;
  const upd = (x: number, y: number) => {
    has = true;
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
  };

  for (const w of ast.wires) {
    upd(w.x1, w.y1);
    upd(w.x2, w.y2);
  }
  for (const f of ast.flags) {
    upd(f.x, f.y);
    if (flagIsGround(f.name)) {
      // Erd-Symbol erstreckt sich 18 LE unter die Flag-Position (Task 10).
      if (f.y + GROUND_H > maxY) maxY = f.y + GROUND_H;
    }
  }
  for (const b of ast.bustaps) {
    upd(b.x, b.y);
  }
  for (const s of ast.symbols) {
    upd(s.x - PLACEHOLDER_HALF_W, s.y - PLACEHOLDER_HALF_H);
    upd(s.x + PLACEHOLDER_HALF_W, s.y + PLACEHOLDER_HALF_H);
    for (const w of s.windows) {
      // Task 11: Window-Label-Positionen sind Content (keine Clipping).
      const p = toWorld(s, w.x, w.y);
      upd(p.x, p.y);
    }
  }
  for (const t of ast.texts) {
    upd(t.x, t.y);
  }
  for (const g of ast.graphics) {
    graphicBBox(g, (x: number, y: number) => upd(x, y));
  }

  if (!has) return null;
  return { minX, minY, maxX, maxY };
}

/** viewBox-String: Content-Box ±10, sonst SHEET-Größe, sonst 100×100. */
export function viewBoxFor(ast: Schematic): string {
  const box = contentBox(ast);
  if (box) {
    const x = box.minX - 10;
    const y = box.minY - 10;
    const w = box.maxX - box.minX + 20;
    const h = box.maxY - box.minY + 20;
    return `${x} ${y} ${w} ${h}`;
  }
  if (ast.sheet) {
    return `0 0 ${ast.sheet.width} ${ast.sheet.height}`;
  }
  return '0 0 100 100';
}

// ---------------------------------------------------------------------------
// Transformationen
// ---------------------------------------------------------------------------

/**
 * Orientation → lokale Koordinaten-Matrix `[a b c d]` für
 * `Welt = (x + a·px + b·py, y + c·px + d·py)`. Empirisch abgeleitet
 * (docs "Geometrie"), identisch für aufgelöste .asy und Fallback-Symbole.
 */
const ORIENT_MATRIX: Record<Orientation, [number, number, number, number]> = {
  R0: [1, 0, 0, 1],
  R90: [0, -1, 1, 0],
  R180: [-1, 0, 0, -1],
  R270: [0, 1, -1, 0],
  M0: [-1, 0, 0, 1],
  M90: [0, 1, 1, 0],
  M180: [1, 0, 0, -1],
  M270: [0, -1, -1, 0],
};

/** Lokale Symbol-Koordinate `(px,py)` → Welt-`(x,y)` inkl. Positions-Verschiebung. */
function toWorld(s: Symbol, px: number, py: number): { x: number; y: number } {
  const [a, b, c, d] = ORIENT_MATRIX[s.orientation];
  return { x: s.x + a * px + b * py, y: s.y + c * px + d * py };
}

/** SVG-`transform` des Symbol-Gruppen-Elements (Grafik + Pins). */
function symbolTransform(s: Symbol): string {
  const rot: Record<Orientation, string> = {
    R0: '',
    R90: 'rotate(90)',
    R180: 'rotate(180)',
    R270: 'rotate(270)',
    M0: '',
    M90: 'rotate(-90)',
    M180: 'rotate(-180)',
    M270: 'rotate(-270)',
  };
  const parts = [`translate(${s.x} ${s.y})`, rot[s.orientation]];
  if (s.orientation[0] === 'M') parts.push('scale(-1 1)');
  return parts.filter(Boolean).join(' ');
}

/**
 * Gegenrotation der Text-Wrapper (Task 10 Upright-Text): Inverses der
 * Orientation-Rotation, damit Text lesbar bleibt. `M`-Spiegelung ohne
 * Rotation → reine Spiegelung; `M` mit Rotation → Matrix.
 */
const COUNTER_ROT: Record<Orientation, string> = {
  R0: '',
  R90: 'rotate(-90)',
  R180: 'rotate(-180)',
  R270: 'rotate(-270)',
  M0: 'scale(-1 1)',
  M90: 'matrix(0 1 1 0 0 0)',
  M180: 'scale(1 -1)',
  M270: 'matrix(0 -1 -1 0 0 0)',
};

/** Text-Wrapper `transform` an lokaler Position `(lx,ly)` im Symbol-Local-Frame. */
function textWrapper(
  s: Symbol,
  lx: number,
  ly: number,
  vertical: boolean
): string {
  const parts = [
    `translate(${lx} ${ly})`,
    COUNTER_ROT[s.orientation],
    vertical ? 'rotate(-90)' : '',
  ];
  return parts.filter(Boolean).join(' ');
}

// ---------------------------------------------------------------------------
// SVG-Element-Helfer
// ---------------------------------------------------------------------------

function el(
  tag: string,
  attrs: Record<string, string>,
  text?: string
): SVGElement {
  const e = document.createElementNS(SVG_NS, tag);
  for (const k of Object.keys(attrs)) {
    e.setAttribute(k, attrs[k]);
  }
  if (text !== undefined) e.textContent = text;
  return e;
}

/**
 * Font-Attribute aus der Config: `font-family`, `font-size`, `weight`.
 * Gewicht: `bold`-Override wenn gesetzt (Task 11: SpiceLine/`.asy`-TEXT
 * non-bold), sonst Config `fontBold`.
 */
function fontAttrs(cfg: RenderConfig, bold?: boolean): Record<string, string> {
  const a: Record<string, string> = {
    'font-family': cfg.fontFamily,
    'font-size': String(cfg.fontSize),
    'font-weight': (bold === undefined ? cfg.fontBold : bold) ? 'bold' : 'normal',
  };
  return a;
}

/** `.asc`-TEXT Größe: `fontSize·size/3` gerundet auf 1 Dezimale (Task 10). */
function ascTextSize(cfg: RenderConfig, size: number): string {
  const v = (cfg.fontSize * size) / 3;
  return String(Math.round(v * 10) / 10);
}

/** Strich-Attribute je LineStyle (Bold = 2×, Config-Strichstärke/Farben). */
function lineAttrs(
  style: LineStyle,
  cfg: RenderConfig,
  color: string
): Record<string, string> {
  const thickness = style === 'Bold' ? cfg.penThickness * 2 : cfg.penThickness;
  const dash =
    style === 'Dashed' ? '6 3' : style === 'Dotted' ? '2 2' : 'none';
  return {
    stroke: color,
    'stroke-width': String(thickness),
    'stroke-dasharray': dash,
    fill: 'none',
  };
}

/** Bounding-Box einer Grafik-Primitive (für Content-Box). */
function graphicBBox(
  g: GraphicPrimitive,
  upd: (x: number, y: number) => void
): void {
  const p =
    (g as { Line?: { x1: number; y1: number; x2: number; y2: number } }).Line ||
    (g as { Rectangle?: { x1: number; y1: number; x2: number; y2: number } })
      .Rectangle ||
    (g as { Circle?: { x1: number; y1: number; x2: number; y2: number } }).Circle;
  const arc = (g as { Arc?: { x1: number; y1: number; x2: number; y2: number } })
    .Arc;
  if (p) {
    upd(Math.min(p.x1, p.x2), Math.min(p.y1, p.y2));
    upd(Math.max(p.x1, p.x2), Math.max(p.y1, p.y2));
  }
  if (arc) {
    const rx = Math.abs(arc.x2 - arc.x1) / 2;
    const ry = Math.abs(arc.y2 - arc.y1) / 2;
    upd(arc.x1 - rx, arc.y1 - ry);
    upd(arc.x1 + rx, arc.y1 + ry);
  }
}

/** .asy-TEXT-Justification → SVG `text-anchor`: `Left*`→start, `Center*`→middle, `Right*`→end. */
function anchorFor(justification: string): 'start' | 'middle' | 'end' {
  const j = justification.toUpperCase();
  if (j.startsWith('CENTER') || j.startsWith('V-CENTER')) return 'middle';
  if (j.startsWith('RIGHT')) return 'end';
  return 'start';
}

/** Task 11: Window-Slot auflösen — Instanz-`WINDOW` zuerst, dann `WINDOW` im `.asy`. */
function windowFor(
  s: Symbol,
  asy: AsySymbol,
  slot: number
): WindowSpec | undefined {
  return (
    s.windows.find((w) => w.index === slot) ??
    asy.windows.find((w) => w.index === slot)
  );
}

/**
 * Window-Justification → SVG `text-anchor` (edge-Modell, Task 14):
 * Plain-Horizontal-Tokens benennen die Textkante am Ankerpunkt
 * (`Left` → rechte Kante am Anker = `end`, `Right` → linke Kante =
 * `start`), `V`-Tokens die Welt-Kante des Textblocks (`VLeft` → `start`,
 * `VRight` → `end`). `Top`/`VTop`, `Bottom`/`VBottom` und `Center`/
 * `VCenter` sind dadurch horizontal zentriert (`middle`).
 */
function windowAnchor(name: string): 'start' | 'middle' | 'end' {
  const j = name.toUpperCase();
  if (j.includes('TOP') || j.includes('BOTTOM') || j.includes('CENTER')) {
    return 'middle';
  }
  const isV = j.startsWith('V');
  const right = j.includes('RIGHT');
  if (isV) return right ? 'end' : 'start';
  return right ? 'start' : 'end';
}

/**
 * Window-Justification → SVG `dominant-baseline`: `Top`/`VTop` → Text über
 * dem Anker (`text-after-edge`), `Bottom`/`VBottom` → Text unter dem Anker
 * (`text-before-edge`), sonst `central`.
 */
function windowBaseline(name: string): string {
  const j = name.toUpperCase();
  if (j === 'TOP' || j === 'VTOP') return 'text-after-edge';
  if (j === 'BOTTOM' || j === 'VBOTTOM') return 'text-before-edge';
  return 'central';
}

/** Window `Invisible` → Slot-Label wird nicht gerendert (kein Fallback). */
function windowInvisible(name: string): boolean {
  return name.trim().toUpperCase() === 'INVISIBLE';
}

// ---------------------------------------------------------------------------
// Einzel-Elemente
// ---------------------------------------------------------------------------

/** Wire → `<line>` (wireColor). */
function wireEl(w: Wire, cfg: RenderConfig): SVGElement {
  const attrs: Record<string, string> = {
    x1: String(w.x1),
    y1: String(w.y1),
    x2: String(w.x2),
    y2: String(w.y2),
  };
  Object.assign(attrs, lineAttrs('Normal', cfg, cfg.wireColor));
  return el('line', attrs);
}

/**
 * FLAG → Erd-Symbol (Name `0`) oder Netlabel-Text (Task 10).
 */
function flagEl(name: string, x: number, y: number, cfg: RenderConfig): SVGElement {
  if (flagIsGround(name)) {
    const g = document.createElementNS(SVG_NS, 'g');
    const a = lineAttrs('Normal', cfg, cfg.symbolColor);
    // Stange: (x,y) → (x,y+18)
    g.appendChild(el('line', { ...a, x1: String(x), y1: String(y), x2: String(x), y2: String(y + GROUND_H) }));
    // Drei abnehmende Balken (Halbweiten 9/6/3), y+6, y+12, y+18.
    const bars: Array<[number, number]> = [
      [y + 6, 9],
      [y + 12, 6],
      [y + 18, 3],
    ];
    for (const [by, hw] of bars) {
      g.appendChild(
        el('line', { ...a, x1: String(x - hw), y1: String(by), x2: String(x + hw), y2: String(by) })
      );
    }
    return g;
  }
  // Netlabel: Name als Text in netlabelColor.
  return el(
    'text',
    {
      ...fontAttrs(cfg),
      x: String(x),
      y: String(y),
      fill: cfg.netlabelColor,
      'text-anchor': 'middle',
      'dominant-baseline': 'central',
      'pointer-events': 'none',
    },
    name
  );
}

/** BUSTAP → gefülltes 4×4-Quadrat (pinColor). */
function bustapEl(x: number, y: number, cfg: RenderConfig): SVGElement {
  return el('rect', {
    x: String(x - 2),
    y: String(y - 2),
    width: '4',
    height: '4',
    fill: cfg.pinColor,
    stroke: 'none',
  });
}

/** `.asc`-TEXT → `<text>` (Größenregel Task 10, textColor). */
function ascTextEl(t: TextDirective, cfg: RenderConfig): SVGElement {
  let text = t.text;
  // Task 11: führendes `;`/`!` wird bei der Anzeige gestrippt (AST unverändert).
  const head = text.charAt(0);
  if (head === ';' || head === '!') text = text.slice(1);
  const anchor =
    t.align.toUpperCase().startsWith('CENTER')
      ? 'middle'
      : t.align.toUpperCase().startsWith('RIGHT')
        ? 'end'
        : 'start';
  return el(
    'text',
    {
      ...fontAttrs(cfg),
      x: String(t.x),
      y: String(t.y),
      fill: cfg.textColor,
      'text-anchor': anchor,
      'dominant-baseline': 'central',
      'font-size': ascTextSize(cfg, t.size),
    },
    text
  );
}

/** .asc-Graphic-Primitive → SVG-Element (symbolColor). */
function graphicEl(g: GraphicPrimitive, cfg: RenderConfig): SVGElement {
  if ('Line' in g) {
    const L = g.Line;
    return el('line', {
      ...lineAttrs(L.style, cfg, cfg.symbolColor),
      x1: String(L.x1),
      y1: String(L.y1),
      x2: String(L.x2),
      y2: String(L.y2),
    });
  }
  if ('Rectangle' in g) {
    const R = g.Rectangle;
    return el('rect', {
      ...lineAttrs(R.style, cfg, cfg.symbolColor),
      x: String(Math.min(R.x1, R.x2)),
      y: String(Math.min(R.y1, R.y2)),
      width: String(Math.abs(R.x2 - R.x1)),
      height: String(Math.abs(R.y2 - R.y1)),
    });
  }
  if ('Circle' in g) {
    const C = g.Circle;
    return el('ellipse', {
      ...lineAttrs(C.style, cfg, cfg.symbolColor),
      cx: String((C.x1 + C.x2) / 2),
      cy: String((C.y1 + C.y2) / 2),
      rx: String(Math.abs(C.x2 - C.x1) / 2),
      ry: String(Math.abs(C.y2 - C.y1) / 2),
    });
  }
  if ('Arc' in g) {
    const A = g.Arc;
    const rx = Math.abs(A.x2 - A.x1) / 2;
    const ry = Math.abs(A.y2 - A.y1) / 2;
    return el('path', {
      ...lineAttrs(A.style, cfg, cfg.symbolColor),
      d: `M ${A.x2} ${A.y2} A ${rx} ${ry} 0 0 1 ${A.x4} ${A.y4}`,
    });
  }
  return document.createElementNS(SVG_NS, 'g');
}

// ---------------------------------------------------------------------------
// Symbole
// ---------------------------------------------------------------------------

/**
 * Text-Wrapper-Gruppierung (Upright, Task 10): `<g transform="translate(lx ly)
 * [Counter] [rotate(-90)]">` plus `<text x=0 y=0>` — Text bleibt bei jeder
 * Orientation lesbar; Grafik/Pins bleiben im Orientation-Transform.
 */
/** Task 11: `uprightText`-Optionen (Gewicht/Baseline-Override). */
interface UprightTextOpts {
  /** Vertikaler Text (`.asy`-`*V`-Justification). */
  vertical?: boolean;
  /** Gewicht-Override (`undefined` = Config `fontBold`). */
  bold?: boolean;
  /** `dominant-baseline` (Default: `central`). */
  baseline?: string;
}

function uprightText(
  s: Symbol,
  lx: number,
  ly: number,
  text: string,
  anchor: 'start' | 'middle' | 'end',
  cfg: RenderConfig,
  opts: UprightTextOpts = {}
): SVGElement {
  const g = document.createElementNS(SVG_NS, 'g');
  g.setAttribute('transform', textWrapper(s, lx, ly, opts.vertical === true));
  const t = el(
    'text',
    {
      ...fontAttrs(cfg, opts.bold),
      x: '0',
      y: '0',
      fill: cfg.textColor,
      'text-anchor': anchor,
      'dominant-baseline': opts.baseline ?? 'central',
    },
    text
  );
  g.appendChild(t);
  return g;
}

/**
 * Aufgelöstes `.asy`-Symbol (Local-Frame): Grafik, Pins, .asy-TEXT,
 * InstName + Value — Upright-Text (Task 10).
 */
function asySymbolEl(
  s: Symbol,
  asy: AsySymbol,
  cfg: RenderConfig
): SVGElement {
  const g = document.createElementNS(SVG_NS, 'g');
  g.setAttribute('transform', symbolTransform(s));

  // Grafik (Local-Frame, Orientation-Transform via Gruppen-Transform).
  for (const gr of asy.graphics) {
    g.appendChild(graphicEl(gr, cfg));
  }

  // .asy-TEXT (upright, Anchor-Mapping; `*V`-Justification → rotiert).
  // Task 11: Gewicht non-bold (Override, Config `fontBold` ignorieren).
  for (const t of asy.texts as AsyText[]) {
    const vertical = t.justification.toUpperCase().includes('V');
    g.appendChild(
      uprightText(
        s,
        t.x,
        t.y,
        t.text,
        anchorFor(t.justification),
        cfg,
        { vertical, bold: false }
      )
    );
  }

  // Window-Slots (Task 11): Slot 0 → InstName, Slot 3 → Value,
  // Slot 39 → SpiceLine. Positionsauflösung pro Slot:
  // Instanz-`WINDOW` → `.asy`-`WINDOW` (InstName ohne Window: Default (0,0)).
  // Auflöstes `Invisible` → Slot-Label wird nicht gerendert (kein Fallback).
  const w0 = windowFor(s, asy, 0);
  if (w0 === undefined || !windowInvisible(w0.name)) {
    g.appendChild(
      uprightText(
        s,
        w0?.x ?? 0,
        w0?.y ?? 0,
        s.attributes['InstName'] ?? '?',
        w0 !== undefined ? windowAnchor(w0.name) : 'start',
        cfg,
        w0 !== undefined ? { baseline: windowBaseline(w0.name) } : {}
      )
    );
  }

  const w3 = windowFor(s, asy, 3);
  if (w3 !== undefined && !windowInvisible(w3.name)) {
    const val = s.attributes['Value'] ?? asy.properties['Value'];
    if (val !== undefined) {
      g.appendChild(
        uprightText(
          s,
          w3.x,
          w3.y,
          val,
          windowAnchor(w3.name),
          cfg,
          { baseline: windowBaseline(w3.name) }
        )
      );
    }
  }

  const w39 = windowFor(s, asy, 39);
  if (w39 !== undefined && !windowInvisible(w39.name)) {
    const spice = s.attributes['SpiceLine'] ?? asy.properties['SpiceLine'];
    if (spice !== undefined) {
      g.appendChild(
        uprightText(
          s,
          w39.x,
          w39.y,
          spice,
          windowAnchor(w39.name),
          cfg,
          { bold: false, baseline: windowBaseline(w39.name) }
        )
      );
    }
  }

  return g;
}

/** Fallback-Box (32×14) + Upright-Label (Task 08/10). */
function fallbackSymbolEl(s: Symbol, cfg: RenderConfig): SVGElement {
  const g = document.createElementNS(SVG_NS, 'g');
  g.setAttribute('transform', symbolTransform(s));
  g.appendChild(
    el('rect', {
      ...lineAttrs('Normal', cfg, cfg.symbolColor),
      x: String(-PLACEHOLDER_HALF_W),
      y: String(-PLACEHOLDER_HALF_H),
      width: String(PLACEHOLDER_HALF_W * 2),
      height: String(PLACEHOLDER_HALF_H * 2),
    })
  );
  const label = s.attributes['InstName'] ?? s.attributes['Value'] ?? '?';
  g.appendChild(uprightText(s, 0, 0, label, 'middle', cfg));
  return g;
}

// ---------------------------------------------------------------------------
// Grid-Punkte + Unverdrahtete Pins (Task 10)
// ---------------------------------------------------------------------------

/**
 * Grid-Punkte = Junctions (LTspice-Abgleich): gefüllte 3×3-Quadrate
 * (gridPointColor), nur bei > 2 Verbindungsenden am exakten Punkt.
 * Verbindungsenden = beide Wire-Endpunkte + aufgelöste Pin-World-Enden
 * (Pins über das Orientation-Mapping); Flag/BUSTAP/Text werden nicht
 * gezählt. 2-Draht-Ecke und Pin+1 Draht (je 2 Enden) → kein Punkt.
 */
function gridPointEls(
  ast: Schematic,
  symbols: Map<string, AsySymbol> | undefined,
  cfg: RenderConfig
): SVGElement[] {
  interface Pt {
    x: number;
    y: number;
    n: number;
  }
  const pts = new Map<string, Pt>();
  const key = (x: number, y: number) => `${Math.round(x * 100)}:${Math.round(y * 100)}`;
  const addEnd = (x: number, y: number) => {
    const k = key(x, y);
    const e = pts.get(k);
    if (e === undefined) {
      pts.set(k, { x, y, n: 1 });
    } else {
      e.n += 1;
    }
  };

  for (const w of ast.wires) {
    addEnd(w.x1, w.y1);
    addEnd(w.x2, w.y2);
  }
  if (symbols !== undefined) {
    for (const s of ast.symbols) {
      const asy = symbols.get(s.name.trim().toLowerCase());
      if (asy === undefined) continue;
      for (const p of asy.pins) {
        const w = toWorld(s, p.x, p.y);
        addEnd(w.x, w.y);
      }
    }
  }

  const els: SVGElement[] = [];
  for (const p of pts.values()) {
    if (p.n <= 2) continue;
    const h = cfg.gridPointSize / 2;
    els.push(
      el('rect', {
        x: String(p.x - h),
        y: String(p.y - h),
        width: String(cfg.gridPointSize),
        height: String(cfg.gridPointSize),
        fill: cfg.gridPointColor,
        stroke: 'none',
      })
    );
  }
  return els;
}

/** Draht-Endpunkt innerhalb von Toleranz (LE) an Punkt `(x,y)`? */
function wireEndpointNear(
  wires: Wire[],
  x: number,
  y: number
): boolean {
  for (const w of wires) {
    if (
      Math.hypot(w.x1 - x, w.y1 - y) <= PIN_TOLERANCE ||
      Math.hypot(w.x2 - x, w.y2 - y) <= PIN_TOLERANCE
    ) {
      return true;
    }
  }
  return false;
}

/**
 * Unverdrahtete Pins (Task 10, Kreise Task 11): gefüllter Kreis (Radius 2,
 * unconnectedPinColor) an jedem Pin-Endpunkt eines aufgelösten .asy-Symbols,
 * an dem kein Wire-Endpunkt innerhalb von PIN_TOLERANCE LE liegt.
 * Pin→Welt über Orientation-Mapping.
 */
function unconnectedPinEls(
  ast: Schematic,
  symbols: Map<string, AsySymbol>,
  cfg: RenderConfig
): SVGElement[] {
  const els: SVGElement[] = [];
  for (const s of ast.symbols) {
    const key = s.name.trim().toLowerCase();
    const asy = symbols.get(key);
    if (!asy) continue;
    for (const p of asy.pins) {
      const w = toWorld(s, p.x, p.y);
      if (!wireEndpointNear(ast.wires, w.x, w.y)) {
        els.push(
          el('circle', {
            cx: String(w.x),
            cy: String(w.y),
            r: '2',
            fill: cfg.unconnectedPinColor,
            stroke: 'none',
          })
        );
      }
    }
  }
  return els;
}

// ---------------------------------------------------------------------------
// Haupt-Renderer
// ---------------------------------------------------------------------------

/**
 * Render den Schematic-AST ins Host-Element (SVG-Rahmen).
 *
 * Reihenfolge: (1) SHEET-Rahmen (bei `showTitleBlocks`, hinten), (2) Wires,
 * (3) Flags (Erd-Symbol/Netlabel), (4) BUSTAPs, (5) `.asc`-TEXTs, (6)
 * `.asc`-Grafik, (7) Symbole (aufgelöst .asy oder Fallback-Box), (8)
 * Grid-Punkte, (9) Unverdrahtete-Pin-Markers.
 *
 * - `symbols`: aufgelöste .asy-Symbole (Key = Basename, case-insensiv);
 *   fehlende Keys → Fallback-Box.
 * - `config`: Render-Konfiguration (Default = `defaultConfig`,
 *   d. h. `render-config.json`).
 */
export function renderSchematic(
  ast: Schematic,
  host: SVGElement,
  symbols?: Map<string, AsySymbol>,
  config: RenderConfig = defaultConfig
): SVGElement {
  const cfg = config;
  host.innerHTML = '';
  host.setAttribute('viewBox', viewBoxFor(ast));

  // (1) SHEET-Rahmen (Task 10): wireColor, hinten, nicht in viewBox.
  if (cfg.showTitleBlocks && ast.sheet) {
    host.appendChild(
      el('rect', {
        ...lineAttrs('Normal', cfg, cfg.wireColor),
        x: '0',
        y: '0',
        width: String(ast.sheet.width),
        height: String(ast.sheet.height),
      })
    );
  }

  // (2) Wires.
  for (const w of ast.wires) host.appendChild(wireEl(w, cfg));

  // (3) Flags (Erd-Symbol oder Netlabel).
  for (const f of ast.flags) host.appendChild(flagEl(f.name, f.x, f.y, cfg));

  // (4) BUSTAPs.
  for (const b of ast.bustaps) host.appendChild(bustapEl(b.x, b.y, cfg));

  // (5) .asc-TEXTs.
  for (const t of ast.texts) host.appendChild(ascTextEl(t, cfg));

  // (6) .asc-Grafik.
  for (const g of ast.graphics) host.appendChild(graphicEl(g, cfg));

  // (7) Symbole (aufgelöst .asy oder Fallback-Box).
  for (const s of ast.symbols) {
    const key = s.name.trim().toLowerCase();
    const asy = symbols?.get(key);
    host.appendChild(asy ? asySymbolEl(s, asy, cfg) : fallbackSymbolEl(s, cfg));
  }

  // (8) Grid-Punkte (nur bei > 2 Verbindungsenden: Wire-Enden + Pins).
  if (cfg.showGridPoints) {
    for (const e of gridPointEls(ast, symbols, cfg)) host.appendChild(e);
  }

  // (9) Unverdrahtete Pins.
  if (cfg.markUnconnectedPins && symbols) {
    for (const e of unconnectedPinEls(ast, symbols, cfg)) host.appendChild(e);
  }

  return host;
}
