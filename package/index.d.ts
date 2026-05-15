export interface ChunkOptions {
  minLength?: number
  maxLength?: number
  phase?: number
  title?: string
}

export interface Chunk {
  level: number
  header: string | null
  headers: Array<string | null>
  breadcrumb: string
  text: string
  length: number
}

/**
 * Chunk markdown text into semantically meaningful pieces.
 *
 * Runs up to three phases depending on `options.phase` (default: 3):
 * - Phase 1: Split at header boundaries, one chunk per paragraph.
 * - Phase 2: Merge adjacent chunks with identical breadcrumbs below `minLength`.
 * - Phase 3: Absorb child sections into parent headers (bottom-up).
 */
export declare function chunk(text: string, options?: ChunkOptions | null): Chunk[]
