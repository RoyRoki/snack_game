/* tslint:disable */
/* eslint-disable */

export class WasmGame {
    free(): void;
    [Symbol.dispose](): void;
    achievement(): string;
    daily_desc(): string;
    daily_done(): boolean;
    daily_progress(): number;
    daily_target(): number;
    danger(): boolean;
    food_timer_secs(): number;
    /**
     * Returns a flat 400-byte grid: 20×20 cells, row-major.
     * Values: 0=empty 1=body 2=head 3=regular 4=bonus 5=golden 6=shrink 7=mystery 8=wall
     */
    grid(): Uint8Array;
    high_scores(): string;
    input(dir: number): void;
    level(): number;
    lives(): number;
    message(): string;
    mode_id(): number;
    constructor(mode: number, diff: number);
    personal_best(): number;
    portal_target(): number;
    restart(): void;
    reversed(): boolean;
    save_score(name: string): void;
    score(): number;
    status(): number;
    streak(): number;
    tick(): void;
    tick_ms(): number;
    time_left(): number;
    toggle_pause(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmgame_free: (a: number, b: number) => void;
    readonly wasmgame_achievement: (a: number) => [number, number];
    readonly wasmgame_daily_desc: (a: number) => [number, number];
    readonly wasmgame_daily_done: (a: number) => number;
    readonly wasmgame_daily_progress: (a: number) => number;
    readonly wasmgame_daily_target: (a: number) => number;
    readonly wasmgame_danger: (a: number) => number;
    readonly wasmgame_food_timer_secs: (a: number) => number;
    readonly wasmgame_grid: (a: number) => [number, number];
    readonly wasmgame_high_scores: (a: number) => [number, number];
    readonly wasmgame_input: (a: number, b: number) => void;
    readonly wasmgame_level: (a: number) => number;
    readonly wasmgame_lives: (a: number) => number;
    readonly wasmgame_message: (a: number) => [number, number];
    readonly wasmgame_mode_id: (a: number) => number;
    readonly wasmgame_new: (a: number, b: number) => number;
    readonly wasmgame_personal_best: (a: number) => number;
    readonly wasmgame_portal_target: (a: number) => number;
    readonly wasmgame_restart: (a: number) => void;
    readonly wasmgame_reversed: (a: number) => number;
    readonly wasmgame_save_score: (a: number, b: number, c: number) => void;
    readonly wasmgame_score: (a: number) => number;
    readonly wasmgame_status: (a: number) => number;
    readonly wasmgame_streak: (a: number) => number;
    readonly wasmgame_tick: (a: number) => void;
    readonly wasmgame_tick_ms: (a: number) => number;
    readonly wasmgame_time_left: (a: number) => number;
    readonly wasmgame_toggle_pause: (a: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
