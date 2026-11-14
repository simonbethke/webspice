import {
  contentBox,
  defaultConfig,
  renderSchematic,
  viewBoxFor,
} from './render';
import {
  AsySymbol,
  Orientation,
  Schematic,
  Symbol,
} from '../types/ast';

const SVG_NS = 'http://www.w3.org/2000/svg';

function schematic(overrides: Partial<Schematic> = {}): Schematic {
  return {
    version: '4',
    sheet: { page: 1, width: 880, height: 680 },
    wires: [],
    flags: [],
    bustaps: [],
    symbols: [],
    texts: [],
    graphics: [],
    ...overrides,
  };
}

function svgHost(): SVGElement {
  return document.createElementNS(SVG_NS, 'svg');
}

/**
 * Test-.asy: Linie x=16 (y 8..96), Pins (4,48)/(28,48), 4 .asy-Texte.
 * Window 3 → (16,52) (Value-Position, wie frühere BBox-Zentrum-Regel).
 */
function asy(): AsySymbol {
  return {
    version: '4',
    symbolType: 'CELL',
    properties: {},
    windows: [{ index: 3, x: 16, y: 52, name: 'Center', visible: 2 }],
    graphics: [{ Line: { x1: 16, y1: 8, x2: 16, y2: 96, style: 'Normal' } }],
    texts: [
      { x: -14, y: 80, justification: 'Left', text: 'lt' },
      { x: 32, y: 48, justification: 'Center', text: 'ct' },
      { x: 32, y: 10, justification: 'Right', text: 'rt' },
      { x: 32, y: 90, justification: 'VLeft', text: 'vt' },
    ],
    pins: [
      { x: 4, y: 48, justification: 'NONE', offset: 0, properties: {} },
      { x: 28, y: 48, justification: 'NONE', offset: 0, properties: {} },
    ],
  };
}

function sym(x: number, y: number, o: Orientation): Symbol {
  return {
    name: 'RES',
    x,
    y,
    orientation: o,
    attributes: { InstName: 'R1', Value: '10k' },
    windows: [],
  };
}

function symbols(): Map<string, AsySymbol> {
  return new Map([['res', asy()]]);
}

/** .asy mit genau einem Pin bei lokalem (20,20) und ohne Windows. */
function indAsy(): AsySymbol {
  return {
    version: '4',
    symbolType: 'CELL',
    properties: {},
    windows: [],
    graphics: [],
    texts: [],
    pins: [{ x: 20, y: 20, justification: 'NONE', offset: 0, properties: {} }],
  };
}

function inds(): Map<string, AsySymbol> {
  return new Map([['ind', indAsy()]]);
}

function indSymbol(instName: string): Symbol {
  return {
    name: 'IND',
    x: 0,
    y: 0,
    orientation: 'R0',
    attributes: { InstName: instName, Value: '1H' },
    windows: [],
  };
}

// Default-Config mit allen drei Toggles aus — für saubere Element-Zählungen.
const noToggles = {
  ...defaultConfig,
  showTitleBlocks: false,
  markUnconnectedPins: false,
  showGridPoints: false,
};

describe('render: Wires, Flags, BUSTAP', () => {
  it('render Wires in wireColor und penThickness', () => {
    const ast = schematic({ wires: [{ x1: 0, y1: 10, x2: 100, y2: 10 }] });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const lines = Array.from(svg.querySelectorAll('line'));
    expect(lines).toHaveLength(1);
    expect(lines[0]).toEqualAttribute('x1', '0');
    expect(lines[0]).toEqualAttribute('y1', '10');
    expect(lines[0]).toEqualAttribute('x2', '100');
    expect(lines[0]).toEqualAttribute('y2', '10');
    expect(lines[0]).toEqualAttribute('stroke', '#0000C0');
    expect(lines[0]).toEqualAttribute('stroke-width', '1');
    expect(lines[0]).toEqualAttribute('fill', 'none');
  });

  it('render benannte FLAGs als Netlabel-Text (netlabelColor)', () => {
    const ast = schematic({ flags: [{ name: 'VCC', x: 20, y: 30 }] });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts).toHaveLength(1);
    expect(texts[0].textContent).toBe('VCC');
    expect(texts[0]).toEqualAttribute('x', '20');
    expect(texts[0]).toEqualAttribute('y', '30');
    expect(texts[0]).toEqualAttribute('fill', '#0000C0');
    expect(texts[0]).toEqualAttribute('font-weight', 'bold');
    expect(texts[0]).toEqualAttribute('font-family', 'Tahoma');
    expect(texts[0]).toEqualAttribute('font-size', '10');
    expect(texts[0]).toEqualAttribute('text-anchor', 'middle');
    expect(Array.from(svg.querySelectorAll('circle'))).toHaveLength(0);
  });

  it('render .asc-Grafik-Linien mit Style-Strichen (symbolColor)', () => {
    const ast = schematic({
      graphics: [
        { Line: { x1: 0, y1: 0, x2: 100, y2: 0, style: 'Normal' } },
        { Line: { x1: 0, y1: 10, x2: 100, y2: 10, style: 'Bold' } },
        { Line: { x1: 0, y1: 20, x2: 100, y2: 20, style: 'Dashed' } },
        { Line: { x1: 0, y1: 30, x2: 100, y2: 30, style: 'Dotted' } },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const lines = Array.from(svg.querySelectorAll('line'));
    expect(lines).toHaveLength(4);
    lines.forEach((l) => {
      expect(l).toEqualAttribute('stroke', '#0000C0');
      expect(l).toEqualAttribute('fill', 'none');
    });
    expect(lines[0]).toEqualAttribute('stroke-dasharray', 'none');
    expect(lines[0]).toEqualAttribute('stroke-width', '1');
    expect(lines[1]).toEqualAttribute('stroke-dasharray', 'none');
    expect(lines[1]).toEqualAttribute('stroke-width', '2');
    expect(lines[2]).toEqualAttribute('stroke-dasharray', '6 3');
    expect(lines[2]).toEqualAttribute('stroke-width', '1');
    expect(lines[3]).toEqualAttribute('stroke-dasharray', '2 2');
    expect(lines[3]).toEqualAttribute('stroke-width', '1');
  });

  it('render BUSTAP als 4x4-Vierkant in pinColor', () => {
    const ast = schematic({ bustaps: [{ x: 10, y: 20 }] });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const rect = svg.querySelector('rect');
    expect(rect).not.toBeNull();
    expect(rect).toEqualAttribute('x', '8');
    expect(rect).toEqualAttribute('y', '18');
    expect(rect).toEqualAttribute('width', '4');
    expect(rect).toEqualAttribute('height', '4');
    expect(rect).toEqualAttribute('fill', '#0000C0');
    expect(rect).toEqualAttribute('stroke', 'none');
    expect(Array.from(svg.querySelectorAll('line'))).toHaveLength(0);
    expect(Array.from(svg.querySelectorAll('text'))).toHaveLength(0);
  });
});

describe('render: .asc TEXT', () => {
  it('render TEXT in textColor mit Anchor-Mapping und Groessenregel', () => {
    const ast = schematic({
      texts: [
        { text: 'A', x: 10, y: 20, align: 'Left', size: 3, is_command: false },
        { text: 'B', x: 30, y: 30, align: 'Center', size: 10, is_command: false },
        { text: 'C', x: 50, y: 40, align: 'Right', size: 2, is_command: false },
        { text: 'D', x: 70, y: 50, align: 'Left', size: 4, is_command: false },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts).toHaveLength(4);
    texts.forEach((t) => {
      expect(t).toEqualAttribute('fill', '#000000');
      expect(t).toEqualAttribute('font-weight', 'bold');
      expect(t).toEqualAttribute('font-family', 'Tahoma');
    });
    expect(texts[0]).toEqualAttribute('x', '10');
    expect(texts[0]).toEqualAttribute('text-anchor', 'start');
    expect(texts[0]).toEqualAttribute('font-size', '10');
    expect(texts[1]).toEqualAttribute('text-anchor', 'middle');
    expect(texts[1]).toEqualAttribute('font-size', '33.3');
    expect(texts[2]).toEqualAttribute('text-anchor', 'end');
    expect(texts[2]).toEqualAttribute('font-size', '6.7');
    expect(texts[3]).toEqualAttribute('font-size', '13.3');
  });
});

describe('render: Symbole (aufgelöst .asy und Fallback)', () => {
  it('render .asy-Geometrie und Upright-Labels (keine Pin-Punkte)', () => {
    const ast = schematic({ symbols: [sym(0, 0, 'R0')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), noToggles);

    const group = svg.firstElementChild as SVGGElement;
    expect(group).toEqualAttribute('transform', 'translate(0 0)');
    const line = group.querySelector('line');
    expect(line).not.toBeNull();
    expect(line).toEqualAttribute('stroke', '#0000C0');

    // Keine Pin-Punkte in der Symbol-Darstellung (LTspice-Abgleich);
    // Pins fließen nur in die Junction- und Unverdrahtet-Regel ein.
    expect(Array.from(group.querySelectorAll('circle'))).toHaveLength(0);

    // 4 .asy-Texte + InstName + Value
    const texts = Array.from(group.querySelectorAll('text'));
    expect(texts).toHaveLength(6);
    const byText = (content: string) =>
      texts.find((t) => t.textContent === content) as SVGTextElement;

    const value = byText('10k');
    expect(value.parentElement).toEqualAttribute('transform', 'translate(16 52)');
    expect(value).toEqualAttribute('x', '0');
    expect(value).toEqualAttribute('y', '0');
    expect(value).toEqualAttribute('text-anchor', 'middle');
    expect(value).toEqualAttribute('fill', '#000000');

    const inst = byText('R1');
    expect(inst.parentElement).toEqualAttribute('transform', 'translate(0 0)');
    expect(inst).toEqualAttribute('text-anchor', 'start');

    const vt = byText('vt');
    expect(vt.parentElement).toEqualAttribute(
      'transform',
      'translate(32 90) rotate(-90)',
    );
  });

  it('render fehlende .asy-Symbole als Fallback-Box mit Upright-Label', () => {
    const ast = schematic({ symbols: [sym(40, 10, 'R0')] });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const group = svg.firstElementChild as SVGGElement;
    expect(group).toEqualAttribute('transform', 'translate(40 10)');
    const frame = group.firstElementChild as SVGRectElement;
    expect(frame.tagName).toBe('rect');
    expect(frame).toEqualAttribute('x', '-16');
    expect(frame).toEqualAttribute('y', '-7');
    expect(frame).toEqualAttribute('width', '32');
    expect(frame).toEqualAttribute('height', '14');
    expect(frame).toEqualAttribute('stroke', '#0000C0');
    expect(frame).toEqualAttribute('fill', 'none');

    expect(Array.from(group.querySelectorAll('circle'))).toHaveLength(0);
    const label = group.querySelector('text') as SVGTextElement;
    expect(label.textContent).toBe('R1');
    expect(label.parentElement).toEqualAttribute('transform', 'translate(0 0)');
    expect(label).toEqualAttribute('text-anchor', 'middle');
  });

  it.each([
    ['R0', '', 'translate(16 52)'],
    ['R90', 'rotate(90)', 'translate(16 52) rotate(-90)'],
    ['R180', 'rotate(180)', 'translate(16 52) rotate(-180)'],
    ['R270', 'rotate(270)', 'translate(16 52) rotate(-270)'],
    ['M0', 'scale(-1 1)', 'translate(16 52) scale(-1 1)'],
    ['M90', 'rotate(-90) scale(-1 1)', 'translate(16 52) matrix(0 1 1 0 0 0)'],
    ['M180', 'rotate(-180) scale(-1 1)', 'translate(16 52) scale(1 -1)'],
    ['M270', 'rotate(-270) scale(-1 1)', 'translate(16 52) matrix(0 -1 -1 0 0 0)'],
  ] as Array<[Orientation, string, string]>)(
    'Orientation %s: Grafik/Pins im Orientation-Transform, Value-Text counter-rotiert',
    (o, outer, counter) => {
      const ast = schematic({ symbols: [sym(0, 0, o)] });
      const svg = svgHost();
      renderSchematic(ast, svg, symbols(), noToggles);

      const group = svg.firstElementChild as SVGGElement;
      expect(group).toEqualAttribute('transform', `translate(0 0)${outer ? ` ${outer}` : ''}`);
      const value = Array.from(group.querySelectorAll('text')).find(
        (t) => t.textContent === '10k',
      ) as SVGTextElement;
      expect(value.parentElement).toEqualAttribute('transform', counter);
      expect(value).toEqualAttribute('x', '0');
      expect(value).toEqualAttribute('y', '0');
    },
  );

  it('M180: Text-Transforms ohne 180 / neg-x-Spiegelung (Akzeptanz)', () => {
    const ast = schematic({ symbols: [sym(100, 200, 'M180')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), noToggles);

    const group = svg.firstElementChild as SVGGElement;
    expect(group).toEqualAttribute(
      'transform',
      'translate(100 200) rotate(-180) scale(-1 1)',
    );
    const texts = Array.from(group.querySelectorAll('text'));
    expect(texts.length).toBeGreaterThan(0);
    texts.forEach((text) => {
      const wrapper = text.closest('g[transform]');
      expect(wrapper).not.toBeNull();
      expect(wrapper.querySelector('text')).toBe(text);
      expect((wrapper as SVGGElement).getAttribute('transform')).not.toMatch(
        /180|scale\(-1 /,
      );
    });
    const inst = texts.find((t) => t.textContent === 'R1') as SVGTextElement;
    expect(inst.parentElement).toEqualAttribute(
      'transform',
      'translate(0 0) scale(1 -1)',
    );
  });

  it('M180: vertikaler .asy-Text kombiniert Gegenrotation + rotate(-90)', () => {
    const ast = schematic({ symbols: [sym(0, 0, 'M180')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), noToggles);

    const group = svg.firstElementChild as SVGGElement;
    const vt = Array.from(group.querySelectorAll('text')).find(
      (t) => t.textContent === 'vt',
    ) as SVGTextElement;
    expect(vt.parentElement).toEqualAttribute(
      'transform',
      'translate(32 90) scale(1 -1) rotate(-90)',
    );
  });
});

describe('render: Junctions (> 2 Verbindungsenden)', () => {
  it('3-Draht-T (3 Enden) → Punkt; freie Enden → keine', () => {
    const ast = schematic({
      wires: [
        { x1: 0, y1: 10, x2: 50, y2: 10 },
        { x1: 50, y1: 10, x2: 100, y2: 10 },
        { x1: 50, y1: 0, x2: 50, y2: 10 },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('height') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    // Nur (50,10) = T-Knoten; Enden (0,10)/(100,10)/(50,0) bleiben ohne Punkt.
    expect(squares).toHaveLength(1);
    expect(squares[0]).toEqualAttribute('x', '47');
    expect(squares[0]).toEqualAttribute('y', '7');
  });

  it('2-Draht-Ecke (2 Enden) → kein Punkt', () => {
    const ast = schematic({
      wires: [
        { x1: 0, y1: 0, x2: 100, y2: 0 },
        { x1: 100, y1: 0, x2: 100, y2: 50 },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('height') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    // (100,0) = Ecke mit genau 2 Enden → kein Punkt; freie Enden auch nicht.
    expect(squares).toHaveLength(0);
  });

  it('markiert gerade Durchläufe und freie Enden nicht', () => {
    const ast = schematic({
      wires: [
        { x1: 0, y1: 10, x2: 50, y2: 10 },
        { x1: 50, y1: 10, x2: 100, y2: 10 },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });
    expect(Array.from(svg.querySelectorAll('rect'))).toHaveLength(0);
  });

  it('Kreuzung (4 Enden) → Punkt; freie Enden → keine', () => {
    const ast = schematic({
      wires: [
        { x1: 0, y1: 10, x2: 50, y2: 10 },
        { x1: 100, y1: 10, x2: 50, y2: 10 },
        { x1: 50, y1: 0, x2: 50, y2: 10 },
        { x1: 50, y1: 60, x2: 50, y2: 10 },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    expect(squares).toHaveLength(1);
    expect(squares[0]).toEqualAttribute('x', '47');
    expect(squares[0]).toEqualAttribute('y', '7');
  });

  it('Pin + Pin + Draht (3 Enden) → Punkt; freies Draht-Ende nicht', () => {
    const ast = schematic({
      wires: [{ x1: 20, y1: 20, x2: 100, y2: 100 }],
      symbols: [indSymbol('L1'), indSymbol('L2')],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, inds(), {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    // (20,20): 2× Pin + 1× Draht = 3 Enden → Punkt; (100,100): 1 Ende → keiner.
    expect(squares).toHaveLength(1);
    expect(squares[0]).toEqualAttribute('x', '17');
    expect(squares[0]).toEqualAttribute('y', '17');
  });

  it('Pin + 1 Draht (2 Enden) → kein Punkt', () => {
    const ast = schematic({
      wires: [{ x1: 20, y1: 20, x2: 100, y2: 100 }],
      symbols: [indSymbol('L1')],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, inds(), {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    expect(squares).toHaveLength(0);
  });

  it('Flag/BUSTAP/Text am Draht-Ende zählen nicht als Enden', () => {
    const ast = schematic({
      wires: [{ x1: 0, y1: 0, x2: 50, y2: 50 }],
      flags: [{ name: 'VCC', x: 50, y: 50 }],
      bustaps: [{ x: 50, y: 50 }],
      texts: [
        { text: 'T', x: 50, y: 50, align: 'Left', size: 3, is_command: false },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
    });

    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return (
        r.getAttribute('width') === '6' &&
        r.getAttribute('fill') === '#0000C0' &&
        r.getAttribute('stroke') === 'none'
      );
    });
    expect(squares).toHaveLength(0);

    // BUSTAP bleibt sichtbar (4×4, pinColor), zählt aber nicht als Ende.
    const bustap = Array.from(svg.querySelectorAll('rect')).find((r) => {
      return (
        r.getAttribute('width') === '4' && r.getAttribute('fill') === '#0000C0'
      );
    }) as SVGRectElement;
    expect(bustap).toEqualAttribute('x', '48');
    expect(bustap).toEqualAttribute('y', '48');
  });

  it('showGridPoints: false → keine Grid-Punkte', () => {
    const ast = schematic({
      wires: [
        { x1: 0, y1: 10, x2: 50, y2: 10 },
        { x1: 50, y1: 10, x2: 100, y2: 10 },
        { x1: 50, y1: 0, x2: 50, y2: 10 },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    expect(Array.from(svg.querySelectorAll('rect'))).toHaveLength(0);
  });
});

describe('render: unverdrahtete Pins', () => {
  it('mark nur unverbundene Pin-Endpunkte (Weltkoordinaten)', () => {
    const ast = schematic({
      wires: [{ x1: 4, y1: 48, x2: 100, y2: 48 }],
      symbols: [sym(0, 0, 'R0')],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), {
      ...defaultConfig,
      showTitleBlocks: false,
      showGridPoints: false,
    });

    const markers = Array.from(svg.querySelectorAll('circle')).filter((c) => {
      return c.getAttribute('fill') === '#C00000';
    });
    // Pin (4,48) = Wire-Endpunkt → kein Marker; Pin (28,48) → Kreis r=2.
    expect(markers).toHaveLength(1);
    expect(markers[0]).toEqualAttribute('cx', '28');
    expect(markers[0]).toEqualAttribute('cy', '48');
    expect(markers[0]).toEqualAttribute('r', '2');
    expect(markers[0]).toEqualAttribute('stroke', 'none');
  });

  it('M180: Pins über Orientation-Mapping in Weltkoordinaten', () => {
    const ast = schematic({ symbols: [sym(0, 0, 'M180')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), {
      ...defaultConfig,
      showTitleBlocks: false,
      showGridPoints: false,
    });

    const markers = Array.from(svg.querySelectorAll('circle')).filter((c) => {
      return c.getAttribute('fill') === '#C00000';
    });
    expect(markers).toHaveLength(2);
    const at = (x: number, y: number) =>
      markers.find(
        (m) =>
          Number(m.getAttribute('cx')) === x &&
          Number(m.getAttribute('cy')) === y,
      );
    expect(at(4, -48)).not.toBeNull();
    expect(at(28, -48)).not.toBeNull();
  });

  it('markUnconnectedPins: false → keine Marker', () => {
    const ast = schematic({ symbols: [sym(0, 0, 'R0')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), noToggles);
    expect(
      Array.from(svg.querySelectorAll('circle')).filter(
        (c) => c.getAttribute('fill') === '#C00000',
      ),
    ).toHaveLength(0);
  });
});

describe('render: Erd-Symbol (FLAG 0)', () => {
  it('render Stange + 3 abnehmende Balken in symbolColor', () => {
    const ast = schematic({ flags: [{ name: '0', x: 10, y: 20 }] });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);

    const lines = Array.from(svg.querySelectorAll('line'));
    expect(lines).toHaveLength(4);
    lines.forEach((l) => {
      expect(l).toEqualAttribute('stroke', '#0000C0');
      expect(l).toEqualAttribute('stroke-width', '1');
    });
    const x1 = (v: number) => lines.find((l) => l.getAttribute('x1') === String(v));
    // Stange: (10,20)→(10,38)
    expect(x1(10)).toEqualAttribute('y1', '20');
    expect(x1(10)).toEqualAttribute('y2', '38');
    // Balken (Halbweite 9): (1,26)→(19,26)
    expect(x1(1)).toEqualAttribute('y1', '26');
    expect(x1(1) as SVGLineElement).toEqualAttribute('x2', '19');
    // Balken (Halbweite 6): (4,32)→(16,32)
    expect(x1(4)).toEqualAttribute('y1', '32');
    expect(x1(4) as SVGLineElement).toEqualAttribute('x2', '16');
    // Balken (Halbweite 3): (7,38)→(13,38)
    expect(x1(7)).toEqualAttribute('y1', '38');
    expect(x1(7) as SVGLineElement).toEqualAttribute('x2', '13');
    expect(Array.from(svg.querySelectorAll('text'))).toHaveLength(0);
  });

  it('contentBox erweitert Erd-Symbol um 18 LE nach unten', () => {
    const ast = schematic({ flags: [{ name: '0', x: 10, y: 20 }] });
    expect(contentBox(ast)).toStrictEqual({
      minX: 10,
      minY: 20,
      maxX: 10,
      maxY: 38,
    });
  });
});

describe('render: Blatt-Rahmen (SHEET)', () => {
  it('showTitleBlocks: Rahmen zuerst, wireColor, außerhalb der viewBox', () => {
    const ast = schematic({ symbols: [sym(400, 340, 'R0')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), {
      ...defaultConfig,
      markUnconnectedPins: false,
      showGridPoints: false,
    });

    const frame = svg.firstElementChild as SVGRectElement;
    expect(frame.tagName).toBe('rect');
    expect(frame).toEqualAttribute('x', '0');
    expect(frame).toEqualAttribute('y', '0');
    expect(frame).toEqualAttribute('width', '880');
    expect(frame).toEqualAttribute('height', '680');
    expect(frame).toEqualAttribute('stroke', '#0000C0');
    expect(frame).toEqualAttribute('stroke-width', '1');
    expect(frame).toEqualAttribute('fill', 'none');
    // viewBox = Content-Box ±10, SHEET-Rahmen bleibt außen.
    expect(svg).toEqualAttribute('viewBox', '374 323 52 34');
  });

  it('showTitleBlocks: false → kein Rahmen', () => {
    const ast = schematic({});
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    expect(Array.from(svg.querySelectorAll('rect'))).toHaveLength(0);
  });
});

describe('render: contentBox / viewBox', () => {
  it('contentBox null bei leerem AST; SHEET → viewBox SHEET-Größe', () => {
    const ast = schematic({});
    expect(contentBox(ast)).toBeNull();
    expect(viewBoxFor(ast)).toBe('0 0 880 680');
  });

  it('leerer AST ohne SHEET → 100x100', () => {
    const ast = { ...schematic({}), sheet: null };
    expect(viewBoxFor(ast)).toBe('0 0 100 100');
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    expect(svg).toEqualAttribute('viewBox', '0 0 100 100');
  });

  it('Wire-Content-Box ±10 LE viewBox', () => {
    const ast = schematic({
      wires: [{ x1: 10, y1: 10, x2: 100, y2: 50 }],
    });
    expect(contentBox(ast)).toStrictEqual({
      minX: 10,
      minY: 10,
      maxX: 100,
      maxY: 50,
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    expect(svg).toEqualAttribute('viewBox', '0 0 110 60');
  });

  it('FLAG-Positionen in der Content-Box (Netlabel)', () => {
    expect(
      contentBox(schematic({ flags: [{ name: 'VCC', x: 20, y: 30 }] })),
    ).toStrictEqual({ minX: 20, minY: 30, maxX: 20, maxY: 30 });
  });

  it('Symbol-Box: 32x14 um den Anchor (±16/±7)', () => {
    expect(contentBox(schematic({ symbols: [sym(10, 20, 'R0')] }))).toStrictEqual({
      minX: -6,
      minY: 13,
      maxX: 26,
      maxY: 27,
    });
  });

  it('viewBox um Netlabels ohne SHEET (±10)', () => {
    const ast = {
      ...schematic({
        flags: [
          { name: 'GND', x: 1, y: 1 },
          { name: 'NC', x: 41, y: 19 },
        ],
      }),
      sheet: null,
    };
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts.map((t) => t.textContent).sort()).toEqual(['GND', 'NC']);
    expect(Array.from(svg.querySelectorAll('circle'))).toHaveLength(0);
    expect(svg).toEqualAttribute('viewBox', '-9 -9 60 38');
  });

  it('viewBox um Erd-Symbol ohne SHEET (+18 LE unten)', () => {
    const ast = {
      ...schematic({ flags: [{ name: '0', x: 10, y: 20 }] }),
      sheet: null,
    };
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, noToggles);
    expect(svg).toEqualAttribute('viewBox', '0 10 20 38');
  });
});

describe('Task 10: Render-Konfiguration', () => {
  it('defaultConfig = render-config.json (Tahoma/10/bold/#000000)', () => {
    expect(defaultConfig).toStrictEqual({
      fontFamily: 'Tahoma',
      fontSize: 10,
      fontBold: true,
      penThickness: 1,
      wireColor: '#0000C0',
      symbolColor: '#0000C0',
      textColor: '#000000',
      netlabelColor: '#0000C0',
      pinColor: '#0000C0',
      gridPointSize: 6,
      gridPointColor: '#0000C0',
      unconnectedPinColor: '#C00000',
      showTitleBlocks: true,
      markUnconnectedPins: true,
      showGridPoints: true,
    });
  });

  it('ad-hoc-Config steuert Typo, Farben und Toggles im SVG', () => {
    const config = {
      ...defaultConfig,
      fontFamily: 'Verdana',
      fontSize: 20,
      fontBold: false,
      penThickness: 3,
      wireColor: '#123456',
      symbolColor: '#0000C0',
      textColor: '#222222',
      netlabelColor: '#444444',
      pinColor: '#555555',
      gridPointColor: '#666666',
      unconnectedPinColor: '#777777',
    };
    const ast = schematic({
      wires: [
        { x1: 0, y1: 10, x2: 50, y2: 10 },
        { x1: 50, y1: 50, x2: 50, y2: 10 },
        { x1: 50, y1: 10, x2: 50, y2: 100 },
      ],
      flags: [{ name: 'VCC', x: 30, y: 40 }],
      bustaps: [{ x: 80, y: 200 }],
      symbols: [sym(0, 0, 'R0')],
      texts: [
        { text: 'X', x: 80, y: 80, align: 'Left', size: 3, is_command: false },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), config);

    // Blatt-Rahmen: wireColor + penThickness, zuerst gezeichnet.
    const frame = svg.firstElementChild as SVGRectElement;
    expect(frame.tagName).toBe('rect');
    expect(frame).toEqualAttribute('stroke', '#123456');
    expect(frame).toEqualAttribute('stroke-width', '3');

    // Typo: Familie/Größe aus der Config, kein fat.
    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts.length).toBeGreaterThan(0);
    texts.forEach((t) => {
      expect(t).toEqualAttribute('font-family', 'Verdana');
      expect(t.getAttribute('font-weight')).not.toBe('bold');
    });
    const asc = texts.find((t) => t.textContent === 'X') as SVGTextElement;
    expect(asc).toEqualAttribute('font-size', '20');
    expect(asc).toEqualAttribute('fill', '#222222');
    const value = texts.find((t) => t.textContent === '10k') as SVGTextElement;
    expect(value).toEqualAttribute('font-size', '20');
    expect(value).toEqualAttribute('fill', '#222222');

    // Wire: wireColor + 2f. penThickness (Normal=1x)
    const wire = Array.from(svg.querySelectorAll('line')).find((l) => {
      return l.getAttribute('x1') === '0' && l.getAttribute('y1') === '10';
    }) as SVGLineElement;
    expect(wire).toEqualAttribute('stroke', '#123456');
    expect(wire).toEqualAttribute('stroke-width', '3');

    // .asy-Grafik: symbolColor
    const group = Array.from(svg.querySelectorAll('g')).find((g) => {
      return (g.getAttribute('transform') ?? '').startsWith('translate(0 0)');
    }) as SVGGElement;
    expect(group.querySelector('line')).toEqualAttribute('stroke', '#0000C0');

    // Netlabel: netlabelColor
    const netlabel = texts.find((t) => t.textContent === 'VCC') as SVGTextElement;
    expect(netlabel).toEqualAttribute('fill', '#444444');
    expect(netlabel).toEqualAttribute('text-anchor', 'middle');
    expect(netlabel).toEqualAttribute('font-size', '20');

    // Keine Pin-Punkte (LTspice-Abgleich); BUSTAP = 4×4-Vierkant in pinColor.
    expect(
      Array.from(svg.querySelectorAll('circle')).filter(
        (c) => c.getAttribute('fill') === '#555555',
      ),
    ).toHaveLength(0);
    const bustap = Array.from(svg.querySelectorAll('rect')).find(
      (r) => r.getAttribute('fill') === '#555555',
    ) as SVGRectElement;
    expect(bustap).toEqualAttribute('x', '78');
    expect(bustap).toEqualAttribute('y', '198');
    expect(bustap).toEqualAttribute('width', '4');
    expect(bustap).toEqualAttribute('height', '4');
    expect(bustap).toEqualAttribute('stroke', 'none');

    // Junction (T-Knoten (50,10), 3 Enden): gridPointColor
    const square = Array.from(svg.querySelectorAll('rect')).find((r) => {
      return (
        r.getAttribute('fill') === '#666666' &&
        r.getAttribute('width') === '6'
      );
    }) as SVGRectElement;
    expect(square).toEqualAttribute('x', '47');
    expect(square).toEqualAttribute('y', '7');

    // Beide Pins unverdrahtet (Wire-Endpunkte (0,10)/(50,10)): Kreise
    const markers = Array.from(svg.querySelectorAll('circle')).filter((c) => {
      return c.getAttribute('fill') === '#777777';
    });
    expect(markers).toHaveLength(2);
  });

  it('ad-hoc-Config: Toggles aus → Rahmen/Grid/Marker weggreifen', () => {
    const config = {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
      showGridPoints: false,
    };
    const ast = schematic({
      wires: [{ x1: 0, y1: 10, x2: 50, y2: 10 }],
      symbols: [sym(0, 0, 'R0')],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), config);

    expect(Array.from(svg.querySelectorAll('rect'))).toHaveLength(0);
    // Keine Grid-Punkte, keine Pin-Marker, kein Blatt-Rahmen.
    const squares = Array.from(svg.querySelectorAll('rect')).filter((r) => {
      return r.getAttribute('width') === '6';
    });
    expect(squares).toHaveLength(0);
  });

  it('.asc-TEXT Groessenregel klar größer/kleiner als Bauteil-Label', () => {
    const ast = schematic({
      texts: [
        { text: 'big', x: 10, y: 20, align: 'Left', size: 10, is_command: false },
        { text: 'mid', x: 20, y: 20, align: 'Left', size: 4, is_command: false },
        { text: 'small', x: 30, y: 20, align: 'Left', size: 2, is_command: false },
      ],
      symbols: [sym(100, 100, 'R0')],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), {
      ...defaultConfig,
      showTitleBlocks: false,
      markUnconnectedPins: false,
      showGridPoints: false,
    });

    const find = (name: string) =>
      Array.from(svg.querySelectorAll('text')).find(
        (t) => t.textContent === name,
      ) as SVGTextElement;
    // size 4 (13.3) > Bauteil-Label (10) > size 2 (6.7)
    expect(find('big').getAttribute('font-size')).toBe('33.3');
    expect(find('mid').getAttribute('font-size')).toBe('13.3');
    expect(find('small').getAttribute('font-size')).toBe('6.7');
    const label = Array.from(svg.querySelectorAll('text')).find(
      (t) => t.textContent === '10k',
    ) as SVGTextElement;
    expect(label.getAttribute('font-size')).toBe('10');
    expect(Number(find('mid').getAttribute('font-size'))).toBeGreaterThan(
      Number(label.getAttribute('font-size')),
    );
    expect(Number(find('small').getAttribute('font-size'))).toBeLessThan(
      Number(label.getAttribute('font-size')),
    );
  });
});

describe('Task 11: LTspice-treue Labels (Windows)', () => {
  const cfg = {
    ...defaultConfig,
    showTitleBlocks: false,
    markUnconnectedPins: false,
    showGridPoints: false,
  };

  it('InstName Default (0,0); Value am asy-Window; SpiceLine ohne Window weg', () => {
    const ast = schematic({ symbols: [sym(0, 0, 'R0')] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const texts = Array.from(svg.querySelectorAll('text'));
    const inst = texts.find((t) => t.textContent === 'R1') as SVGTextElement;
    expect(inst.parentElement).toEqualAttribute('transform', 'translate(0 0)');
    expect(inst).toEqualAttribute('text-anchor', 'start');
    const val = texts.find((t) => t.textContent === '10k') as SVGTextElement;
    expect(val.parentElement).toEqualAttribute('transform', 'translate(16 52)');
    expect(val).toEqualAttribute('text-anchor', 'middle');
    const spice = texts.find((t) =>
      (t.textContent ?? '').includes('SpiceLine'),
    );
    expect(spice).toBeUndefined();
  });

  it('Instanz-Window überschreibt asy-Window (Position + Anchor)', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R0'),
      windows: [{ index: 3, x: 40, y: 60, name: 'Right', visible: 2 }],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const val = Array.from(svg.querySelectorAll('text')).find(
      (t) => t.textContent === '10k',
    ) as SVGTextElement;
    expect(val.parentElement).toEqualAttribute('transform', 'translate(40 60)');
    expect(val).toEqualAttribute('text-anchor', 'start');
  });

  it('Value-Fallback aus .asy SYMATTR (Value) ohne Instanz-Value', () => {
    const s: Symbol = { ...sym(0, 0, 'R0'), attributes: { InstName: 'R1' } };
    const asyWith = asy();
    asyWith.properties = { Value: 'R5k' };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, new Map([['res', asyWith]]), cfg);

    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts.find((t) => t.textContent === '10k')).toBeUndefined();
    const val = texts.find((t) => t.textContent === 'R5k') as SVGTextElement;
    expect(val.parentElement).toEqualAttribute('transform', 'translate(16 52)');
  });

  it('SpiceLine Slot 39: Anzeige + non-bold + VTop-Baseline', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R0'),
      windows: [{ index: 39, x: 20, y: 30, name: 'VTop', visible: 2 }],
      attributes: {
        InstName: 'R1',
        Value: '10k',
        SpiceLine: '.ac dec 10 1k 100M 10',
      },
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const spice = Array.from(svg.querySelectorAll('text')).find(
      (t) => (t.textContent ?? '').startsWith('.ac'),
    ) as SVGTextElement;
    expect(spice.parentElement).toEqualAttribute('transform', 'translate(20 30)');
    expect(spice).toEqualAttribute('font-weight', 'normal');
    expect(spice).toEqualAttribute('dominant-baseline', 'text-after-edge');
  });

  it('.asc-TEXT: führendes Semikolon/Ausrufezeichen wird gestrippt', () => {
    const ast = schematic({
      texts: [
        {
          text: ';.ac dec 10 1k 100M 10',
          x: 5,
          y: 5,
          align: 'Left',
          size: 3,
          is_command: true,
        },
        {
          text: '!comment',
          x: 5,
          y: 15,
          align: 'Center',
          size: 3,
          is_command: false,
        },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, undefined, cfg);

    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts.map((t) => t.textContent).sort()).toEqual([
      '.ac dec 10 1k 100M 10',
      'comment',
    ]);
  });

  it('Gewicht: InstName/Value/.asc-TEXT bold; SpiceLine/.asy-TEXT non-bold', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R0'),
      windows: [{ index: 39, x: 20, y: 30, name: 'Center', visible: 2 }],
      attributes: { InstName: 'R1', Value: '10k', SpiceLine: 'spice' },
    };
    const ast = schematic({
      symbols: [s],
      texts: [
        { text: 'X', x: 80, y: 80, align: 'Left', size: 3, is_command: false },
      ],
    });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const find = (name: string) =>
      Array.from(svg.querySelectorAll('text')).find(
        (t) => t.textContent === name,
      ) as SVGTextElement;
    expect(find('R1')).toEqualAttribute('font-weight', 'bold');
    expect(find('10k')).toEqualAttribute('font-weight', 'bold');
    expect(find('X')).toEqualAttribute('font-weight', 'bold');
    expect(find('lt')).toEqualAttribute('font-weight', 'normal');
    expect(find('spice')).toEqualAttribute('font-weight', 'normal');
  });

  it('Edge-Modell: Top/Bottom → horizontal zentriert, Baseline je Kante', () => {
    const top: Symbol = {
      ...sym(0, 0, 'R0'),
      attributes: { InstName: 'RT', Value: 'top-val' },
      windows: [{ index: 3, x: 30, y: 40, name: 'Top', visible: 2 }],
    };
    const bot: Symbol = {
      ...sym(0, 100, 'R0'),
      attributes: { InstName: 'RB', Value: 'bot-val' },
      windows: [{ index: 3, x: 30, y: 40, name: 'Bottom', visible: 2 }],
    };
    const ast = schematic({ symbols: [top, bot] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const find = (name: string) =>
      Array.from(svg.querySelectorAll('text')).find(
        (t) => t.textContent === name,
      ) as SVGTextElement;
    const t = find('top-val');
    expect(t.parentElement).toEqualAttribute('transform', 'translate(30 40)');
    expect(t).toEqualAttribute('text-anchor', 'middle');
    expect(t).toEqualAttribute('dominant-baseline', 'text-after-edge');

    const b = find('bot-val');
    expect(b.parentElement).toEqualAttribute('transform', 'translate(30 40)');
    expect(b).toEqualAttribute('text-anchor', 'middle');
    expect(b).toEqualAttribute('dominant-baseline', 'text-before-edge');
  });

  it('VLeft/VCenter/VRight → start/central, middle/central, end/central', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R0'),
      attributes: {
        InstName: 'vl-name',
        Value: 'vc-val',
        SpiceLine: 'vp-line',
      },
      windows: [
        { index: 0, x: 10, y: 10, name: 'VLeft', visible: 2 },
        { index: 3, x: 20, y: 20, name: 'VCenter', visible: 2 },
        { index: 39, x: 30, y: 30, name: 'VRight', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const find = (name: string) =>
      Array.from(svg.querySelectorAll('text')).find(
        (t) => t.textContent === name,
      ) as SVGTextElement;
    const vl = find('vl-name');
    expect(vl.parentElement).toEqualAttribute('transform', 'translate(10 10)');
    expect(vl).toEqualAttribute('text-anchor', 'start');
    expect(vl).toEqualAttribute('dominant-baseline', 'central');

    const vc = find('vc-val');
    expect(vc.parentElement).toEqualAttribute('transform', 'translate(20 20)');
    expect(vc).toEqualAttribute('text-anchor', 'middle');
    expect(vc).toEqualAttribute('dominant-baseline', 'central');

    const vr = find('vp-line');
    expect(vr.parentElement).toEqualAttribute('transform', 'translate(30 30)');
    expect(vr).toEqualAttribute('text-anchor', 'end');
    expect(vr).toEqualAttribute('dominant-baseline', 'central');
    expect(vr).toEqualAttribute('font-weight', 'normal');
  });

  it('Invisible (Instanz) schaltet Slots ab — kein asy-Fallback', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R0'),
      attributes: { InstName: 'R1', Value: '10k' },
      windows: [
        { index: 0, x: 5, y: 5, name: 'Invisible', visible: 1 },
        { index: 3, x: 16, y: 52, name: 'Invisible', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    // asy() trägt Window 3 'Center' — die Instanz-`Invisible` gewinnt:
    // weder InstName (Slot 0) noch Value (Slot 3) erscheinen (kein Fallback).
    const texts = Array.from(svg.querySelectorAll('text'));
    expect(texts.map((t) => t.textContent)).not.toContain('R1');
    expect(texts.map((t) => t.textContent)).not.toContain('10k');
  });

  it('Window-World-Positionen zählen in die contentBox ein', () => {
    // R0: Welt = Anchor + lokal; R90: Matrix [0 -1 1 0] → (x,y)→(−y,x).
    const r0: Symbol = {
      ...sym(100, 100, 'R0'),
      windows: [{ index: 3, x: 40, y: 60, name: 'Left', visible: 2 }],
    };
    expect(contentBox(schematic({ symbols: [r0] }))).toStrictEqual({
      minX: 84,
      minY: 93,
      maxX: 140,
      maxY: 160,
    });

    const r90: Symbol = {
      ...sym(100, 100, 'R90'),
      windows: [{ index: 3, x: 40, y: 60, name: 'Left', visible: 2 }],
    };
    // lokal (40,60) → R90 → (−60,40) → Welt (40,140) → erweitert minX/maxY.
    expect(contentBox(schematic({ symbols: [r90] }))).toStrictEqual({
      minX: 40,
      minY: 93,
      maxX: 116,
      maxY: 140,
    });
  });
});

describe('Task 14: LTspice-Optik-Abgleich 2 (jtk3-Geometrie)', () => {
  const cfg = {
    ...defaultConfig,
    showTitleBlocks: false,
    markUnconnectedPins: false,
    showGridPoints: false,
  };

  const byText = (svg: SVGElement, content: string) =>
    Array.from(svg.querySelectorAll('text')).find(
      (t) => t.textContent === content,
    ) as SVGTextElement;

  it('C30 M0/Left: Name+Value end/central, rechte Textkante am Anker', () => {
    const s: Symbol = {
      ...sym(0, 0, 'M0'),
      attributes: { InstName: 'C30', Value: '39n' },
      windows: [
        { index: 0, x: -51, y: 5, name: 'Left', visible: 2 },
        { index: 3, x: -50, y: 45, name: 'Left', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const name = byText(svg, 'C30');
    expect(name.parentElement).toEqualAttribute(
      'transform',
      'translate(-51 5) scale(-1 1)',
    );
    expect(name).toEqualAttribute('text-anchor', 'end');
    expect(name).toEqualAttribute('dominant-baseline', 'central');

    const val = byText(svg, '39n');
    expect(val.parentElement).toEqualAttribute(
      'transform',
      'translate(-50 45) scale(-1 1)',
    );
    expect(val).toEqualAttribute('text-anchor', 'end');
    expect(val).toEqualAttribute('dominant-baseline', 'central');
  });

  it('D1 R180: Name Left→end, Value VRight→end, beide horizontal', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R180'),
      attributes: { InstName: 'D1', Value: 'BZX84B18VL' },
      windows: [
        { index: 0, x: 24, y: 64, name: 'Left', visible: 2 },
        { index: 3, x: 49, y: -66, name: 'VRight', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const name = byText(svg, 'D1');
    expect(name.parentElement).toEqualAttribute(
      'transform',
      'translate(24 64) rotate(-180)',
    );
    expect(name).toEqualAttribute('text-anchor', 'end');
    expect(name).toEqualAttribute('dominant-baseline', 'central');

    const val = byText(svg, 'BZX84B18VL');
    expect(val.parentElement).toEqualAttribute(
      'transform',
      'translate(49 -66) rotate(-180)',
    );
    expect(val).toEqualAttribute('text-anchor', 'end');
    expect(val).toEqualAttribute('dominant-baseline', 'central');
  });

  it('D2 R270: Name VTop→middle/above, Value VBottom→middle/below', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R270'),
      attributes: { InstName: 'D2', Value: 'BZX84B18VL' },
      windows: [
        { index: 0, x: 36, y: 32, name: 'VTop', visible: 2 },
        { index: 3, x: -4, y: 32, name: 'VBottom', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const name = byText(svg, 'D2');
    expect(name.parentElement).toEqualAttribute(
      'transform',
      'translate(36 32) rotate(-270)',
    );
    expect(name).toEqualAttribute('text-anchor', 'middle');
    expect(name).toEqualAttribute('dominant-baseline', 'text-after-edge');

    const val = byText(svg, 'BZX84B18VL');
    expect(val.parentElement).toEqualAttribute(
      'transform',
      'translate(-4 32) rotate(-270)',
    );
    expect(val).toEqualAttribute('text-anchor', 'middle');
    expect(val).toEqualAttribute('dominant-baseline', 'text-before-edge');
  });

  it('R24 R90: Name VBottom→middle/below, Value VTop→middle/above', () => {
    const s: Symbol = {
      ...sym(0, 0, 'R90'),
      attributes: { InstName: 'R24', Value: '100k' },
      windows: [
        { index: 0, x: 5, y: 56, name: 'VBottom', visible: 2 },
        { index: 3, x: 27, y: 56, name: 'VTop', visible: 2 },
      ],
    };
    const ast = schematic({ symbols: [s] });
    const svg = svgHost();
    renderSchematic(ast, svg, symbols(), cfg);

    const name = byText(svg, 'R24');
    expect(name.parentElement).toEqualAttribute(
      'transform',
      'translate(5 56) rotate(-90)',
    );
    expect(name).toEqualAttribute('text-anchor', 'middle');
    expect(name).toEqualAttribute('dominant-baseline', 'text-before-edge');

    const val = byText(svg, '100k');
    expect(val.parentElement).toEqualAttribute(
      'transform',
      'translate(27 56) rotate(-90)',
    );
    expect(val).toEqualAttribute('text-anchor', 'middle');
    expect(val).toEqualAttribute('dominant-baseline', 'text-after-edge');
  });
});
