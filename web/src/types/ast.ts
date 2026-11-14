/**
 * TypeScript-Abbild des Schematic-ASTs (JSON-Vertrag der WASM-API `parse`).
 * Entspricht exakt den Serialisierungs-Feldern in `src/model.rs`.
 */

/** Seiten-/Canvas-Definition (`SHEET`). */
export interface Sheet {
  page: number;
  width: number;
  height: number;
}

/** Eine Leitung zwischen zwei Punkten (`WIRE`). */
export interface Wire {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/** Named-Netz-Terminal (`FLAG`). */
export interface Flag {
  x: number;
  y: number;
  name: string;
}

/** Bus-Anschlusspunkt (`BUSTAP`). */
export interface Bustap {
  x: number;
  y: number;
}

/** Symbol-Orientierung (LTspice-Notation `R<n>` / `M<n>`). */
export type Orientation = 'R0' | 'R90' | 'R180' | 'R270' | 'M0' | 'M90' | 'M180' | 'M270';

/** Ein Bauteil-Symbol inkl. Attributen. */
export interface Symbol {
  name: string;
  x: number;
  y: number;
  orientation: Orientation;
  attributes: Record<string, string>;
  /** `WINDOW`-Slots der Instanz (`Symbol.windows` im Rust-AST); leer wenn keine. */
  windows: WindowSpec[];
}

/** Text-/Kommando-Anweisung (`TEXT`). */
export interface TextDirective {
  x: number;
  y: number;
  align: string;
  size: number;
  is_command: boolean;
  text: string;
}

/** Strichstil. */
export type LineStyle = 'Normal' | 'Bold' | 'Dashed' | 'Dotted';

interface GraphicBase {
  style: LineStyle;
  /** Optionales numerisches Schluss-Flag der Quellzeile (`.asc`/`.asy`); `null` wenn nicht vorhanden. */
  flag?: number | null;
}

/** Linien-Primitive. */
export interface LineGraphic extends GraphicBase {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/** Rechteck-Primitive (Diagonalendpunkten zugeordnet). */
export interface RectangleGraphic extends GraphicBase {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/** Kreis-/Ellipse-Primitive. */
export interface CircleGraphic extends GraphicBase {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/** Bogen-Primitive. */
export interface ArcGraphic extends GraphicBase {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  x3: number;
  y3: number;
  x4: number;
  y4: number;
}

/**
 * Extern getaggtes Enum: genau eins der Schlüssel ist gesetzt
 * (`{"Line": {…}}`, `{"Rectangle": {…}}`, …).
 */
export type GraphicPrimitive =
  | { Line: LineGraphic }
  | { Rectangle: RectangleGraphic }
  | { Circle: CircleGraphic }
  | { Arc: ArcGraphic };

/** Das komplette Schematic (Wurzel-Objekt des JSON-ASTs). */
export interface Schematic {
  version: string | null;
  sheet: Sheet | null;
  wires: Wire[];
  flags: Flag[];
  bustaps: Bustap[];
  symbols: Symbol[];
  texts: TextDirective[];
  graphics: GraphicPrimitive[];
}

/** Fenster-/Attribut-Anzeigeslot (`WINDOW`), identisch in `.asc` und `.asy`. */
export interface WindowSpec {
  index: number;
  x: number;
  y: number;
  /** Attribut-/Justifikation-TOKEN verbatim (`name`). */
  name: string;
  /** Sichtbarkeits-Flag wie in der Datei. */
  visible: number;
}

/** `.asy`-`TEXT` (Abschnitt „Symbol-Rendering (.asy)", JSON `AsySymbol`). */
export interface AsyText {
  x: number;
  y: number;
  /** LTspice-Justification verbatim (`Left`, `Center`, `VLeft`, …). */
  justification: string;
  /** Textinhalt verbatim. */
  text: string;
}

/** `.asy`-`PIN`; `name` fehlt im JSON, wenn die `PIN`-Zeile keinen Namen hatte. */
export interface AsyPin {
  x: number;
  y: number;
  /** PIN-Justification verbatim (Korpus: `NONE`). */
  justification: string;
  offset: number;
  name?: string;
  properties: Record<string, string>;
}

/**
 * Geparstes LTspice-Symbol (`.asy`); JSON-Vertrag der WASM-API
 * `parse_asy` (Abschnitt „Symbol-Rendering (.asy)").
 */
export interface AsySymbol {
  version: string;
  /** `SYMBOLTYPE`-Token verbatim (`"CELL"`/`"BLOCK"`), `null` wenn nicht vorhanden. */
  symbolType: string | null;
  /** `SYMATTR`-Paare verbatim (`Value`, `Prefix`, …). */
  properties: Record<string, string>;
  windows: WindowSpec[];
  graphics: GraphicPrimitive[];
  texts: AsyText[];
  pins: AsyPin[];
}
