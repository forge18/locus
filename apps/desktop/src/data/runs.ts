import { RUN_ROWS, RUN_STATS } from '../fixtures/runs'
import type { RunRow, RunStat } from '../fixtures/runs'

export { DEFAULT_RANGE, MODEL_TIERS, RANGES, SEARCH_NOTE } from '../fixtures/runs'
export type { RunRow, RunStat } from '../fixtures/runs'

/** How many rows a page carries. One page is what a first paint has to wait for. */
export const PAGE_SIZE = 100

/** Becomes: invoke("runs_list", { filter }) — the whole set, for counts only. */
export function useRuns(): RunRow[] {
  return RUN_ROWS
}

/** Becomes: invoke("runs_count", { filter }) */
export function useRunCount(): number {
  return RUN_ROWS.length
}

/**
 * Becomes: invoke("runs_page", { offset, limit, filter })
 *
 * Rows arrive a page at a time. The table asks for the next one as the window
 * approaches the end of what it has, so a 612-row list costs one page to open
 * rather than 612 rows of DOM nobody has scrolled to.
 */
export function useRunsPage(offset: number, limit = PAGE_SIZE): RunRow[] {
  return RUN_ROWS.slice(offset, offset + limit)
}

/** Becomes: invoke("run_stats", { range }) */
export function useRunStats(): RunStat[] {
  return RUN_STATS
}

/**
 * Virtualization is on, and rows load lazily.
 *
 * The Runs table is drawn at 612 rows and Sessions at 300, and both grow with
 * real data rather than staying at the size the mockup happened to draw. Rendering
 * every row costs ~6,100 DOM nodes for a list nobody has scrolled, and the cost
 * is paid on every filter change — so the table renders a window and asks for the
 * next page as that window approaches the end of what it has.
 *
 * `VirtualTable` composes the ordinary `Column` definitions, so a screen keeps its
 * column types, its mono numerics and its alignment when it switches over.
 */
export const VIRTUALIZATION_NEEDED = true

/** Nodes a fully-rendered table would cost, above which the window pays for itself. */
export const VIRTUALIZATION_THRESHOLD_NODES = 10_000
