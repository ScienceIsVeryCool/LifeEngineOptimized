/* tslint:disable */
/* eslint-disable */
export class Grid {
  free(): void;
  /**
   * Creates a new Grid associated with a canvas element by its ID.
   * `width` and `height` represent the grid dimensions (number of pixels).
   */
  constructor(width: number, height: number, canvas_id: string);
  /**
   * Set the color of a specific pixel in the grid.
   * Color is a 24-bit value in the form 0xRRGGBB.
   */
  set_pixel(x: number, y: number, color: number): void;
  /**
   * Renders the grid on the canvas.
   * `pixel_size` controls how large each pixel appears on the canvas.
   */
  render(pixel_size: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_grid_free: (a: number, b: number) => void;
  readonly grid_new: (a: number, b: number, c: number, d: number) => number;
  readonly grid_set_pixel: (a: number, b: number, c: number, d: number) => void;
  readonly grid_render: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
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
