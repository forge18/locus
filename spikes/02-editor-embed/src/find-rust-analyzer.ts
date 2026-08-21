// rust-analyzer is not reliably on PATH inside a test runner, and in Locus it
// will not be on PATH at all — PLAN.md §LSP makes language servers marketplace
// entries installed into an agent's image, so the host has to resolve one
// rather than assume it.
import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

export function findRustAnalyzer(): string {
  if (process.env.RUST_ANALYZER) return process.env.RUST_ANALYZER;
  for (const candidate of [
    join(process.env.HOME ?? '', '.cargo/bin/rust-analyzer'),
    '/usr/local/bin/rust-analyzer',
    '/opt/homebrew/bin/rust-analyzer',
  ]) if (existsSync(candidate)) return candidate;
  try {
    return execFileSync('rustup', ['which', 'rust-analyzer'], { encoding: 'utf8' }).trim();
  } catch { /* fall through */ }
  return 'rust-analyzer';
}
