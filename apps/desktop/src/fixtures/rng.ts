// schema: none — a helper, not data
// replaced by: nothing; real data arrives from the core
//
// A seeded generator, so a 612-row table is the same 612 rows on every run and a
// screenshot diff means something.

export function rng(seed: number): () => number {
  let s = seed >>> 0
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0
    return s / 0x100000000
  }
}

export function pick<T>(next: () => number, items: readonly T[]): T {
  return items[Math.floor(next() * items.length)]
}

/** Minutes before the fixed "now" the fixtures are written against. */
export const NOW = Date.parse('2026-08-20T14:32:00Z')

export function ago(minutes: number): string {
  return new Date(NOW - minutes * 60_000).toISOString()
}
