/* tslint:disable */
/* eslint-disable */

export class JsSecs1BlockTransfer {
    free(): void;
    [Symbol.dispose](): void;
    constructor(config_json: string);
    poll_event(): string | undefined;
    poll_read(): string | undefined;
    poll_timeout(): string | undefined;
    poll_write(): Uint8Array | undefined;
    read(bytes: Uint8Array): void;
    timeout(key: string): void;
    write(block_json: string): void;
}

export class JsWebRuntime {
    free(): void;
    [Symbol.dispose](): void;
    dataSourceState(): string;
    hasError(): boolean;
    isOpen(): boolean;
    markClosed(): void;
    markFailed(): void;
    markOpen(): void;
    constructor(session_id: number, role: string, on_write: Function, on_open?: Function | null, on_close?: Function | null, on_read_request?: Function | null);
    pendingReadLength(): number;
    pushReadBytes(bytes: Uint8Array): void;
    start(): void;
    tick(): void;
}

export function decode_secs2(bytes: Uint8Array): string;

export function encode_secs2(json: string): Uint8Array;

export function init_web_logger(level: string, callback: Function): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_jswebruntime_free: (a: number, b: number) => void;
    readonly init_web_logger: (a: number, b: number, c: any) => [number, number];
    readonly jswebruntime_dataSourceState: (a: number) => [number, number];
    readonly jswebruntime_hasError: (a: number) => number;
    readonly jswebruntime_isOpen: (a: number) => number;
    readonly jswebruntime_markClosed: (a: number) => void;
    readonly jswebruntime_markFailed: (a: number) => void;
    readonly jswebruntime_markOpen: (a: number) => void;
    readonly jswebruntime_new: (a: number, b: number, c: number, d: any, e: number, f: number, g: number) => [number, number, number];
    readonly jswebruntime_pendingReadLength: (a: number) => number;
    readonly jswebruntime_pushReadBytes: (a: number, b: any) => void;
    readonly jswebruntime_start: (a: number) => [number, number];
    readonly jswebruntime_tick: (a: number) => [number, number];
    readonly decode_secs2: (a: number, b: number) => [number, number, number, number];
    readonly encode_secs2: (a: number, b: number) => [number, number, number, number];
    readonly __wbg_jssecs1blocktransfer_free: (a: number, b: number) => void;
    readonly jssecs1blocktransfer_new: (a: number, b: number) => [number, number, number];
    readonly jssecs1blocktransfer_poll_event: (a: number) => [number, number, number, number];
    readonly jssecs1blocktransfer_poll_read: (a: number) => [number, number, number, number];
    readonly jssecs1blocktransfer_poll_timeout: (a: number) => [number, number];
    readonly jssecs1blocktransfer_poll_write: (a: number) => [number, number];
    readonly jssecs1blocktransfer_read: (a: number, b: number, c: number) => [number, number];
    readonly jssecs1blocktransfer_timeout: (a: number, b: number, c: number) => [number, number];
    readonly jssecs1blocktransfer_write: (a: number, b: number, c: number) => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
