/**
 * Lazy WASM-Loader: lädt das Glue-Modul und das Wasm-Binary zur Laufzeit
 * relativ zur Site-Wurzel (document.baseURI) und initialisiert genau einmal.
 *
 * Das Glue wird bewusst NICHT gebundelt (dynamischer Import einer
 * Laufzeit-URL): so bleibt die Binär-URL stabil unter
 * `<site>/wasm/wasm_bg.wasm`, unabhängig vom Bundle-Standort.
 */

interface WasmApi {
  parse: (bytes: Uint8Array | number[]) => string;
  parse_asy: (bytes: Uint8Array | number[]) => string;
}

interface WasmModule extends WasmApi {
  default?: (input?: unknown) => Promise<unknown>;
}

let ready: Promise<WasmApi> | null = null;

function resolveUrl(pathName: string): string {
  const base = typeof document !== 'undefined' ? document.baseURI : 'http://localhost:8080/';
  return new URL(pathName, base).href;
}

/** Lädt + initialisiert die WASM-API genau einmal (gecacht). */
export function ensureWasm(): Promise<WasmApi> {
  if (ready === null) {
    ready = (async () => {
      const mod = (await import(/* @vite-ignore */ resolveUrl('wasm/wasm.js'))) as WasmModule;
      const init = mod.default ?? (mod as unknown as () => Promise<unknown>);
      await init({ module_or_path: resolveUrl('wasm/wasm_bg.wasm') });
      return mod;
    })();
  }
  return ready;
}

/**
 * Parst die Bytes einer `.asc`-Datei und liefert das Schematic
 * als JSON-String (Wurzel-Objekt, Felder laut `types/ast.ts`).
 */
export function parseAsc(bytes: Uint8Array | number[]): Promise<string> {
  return ensureWasm().then((api) => api.parse(bytes));
}

/**
 * Parst die Bytes einer `.asy`-Datei (LTspice-Symboldefinition) und liefert
 * das `AsySymbol` als JSON-String (Felder laut `types/ast.ts`);
 * Fehler werfen den `Display`-String der WASM-Seite (z. B. `line 5: …`).
 */
export function parseAsy(bytes: Uint8Array | number[]): Promise<string> {
  return ensureWasm().then((api) => api.parse_asy(bytes));
}
