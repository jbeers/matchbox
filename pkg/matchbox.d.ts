/* tslint:disable */
/* eslint-disable */

export class BoxLangVM {
    free(): void;
    [Symbol.dispose](): void;
    call(name: string, args: Array<any>): any;
    load_bytecode(bytes: Uint8Array): void;
    constructor();
}

export function run_boxlang(source: string): string;

export function run_boxlang_bytecode(bytes: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly main: (a: number, b: number) => number;
    readonly __wbg_boxlangvm_free: (a: number, b: number) => void;
    readonly boxlangvm_call: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly boxlangvm_load_bytecode: (a: number, b: number, c: number, d: number) => void;
    readonly boxlangvm_new: () => number;
    readonly run_boxlang: (a: number, b: number, c: number) => void;
    readonly run_boxlang_bytecode: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
