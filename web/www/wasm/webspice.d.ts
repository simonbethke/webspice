/* tslint:disable */
/* eslint-disable */

/**
 * Parst `.asc`-Bytes und liefert das [crate::Schematic] als JSON-String.
 *
 * Fehler (z. B. `line 3: invalid number …`) werden als `Display`-String
 * geworfen; Frontend: `catch (e) { message = String(e) }`.
 */
export function parse(bytes: Uint8Array | number[]): string;

/**
 * Parst `.asy`-Bytes (LTspice-Symboleditor-Datei) und liefert das
 * [crate::AsySymbol] als JSON-String.
 *
 * Fehler (z. B. `line 1: missing required Version line`) werden wie bei
 * [`parse`] als `Display`-String geworfen; Frontend: `catch (e) {
 * message = String(e) }`.
 */
export function parse_asy(bytes: Uint8Array | number[]): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly parse: (a: number, b: number) => [number, number, number, number];
    readonly parse_asy: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
