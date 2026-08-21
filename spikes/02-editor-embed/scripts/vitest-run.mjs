// The task verifies are written `pnpm test -- <name>`, which is the ordinary
// npm idiom for "pass this through". pnpm forwards the literal `--` as well, and
// vitest reads a bare `--` as a positional filter that matches everything — so
// `pnpm test -- reject-nonterminating` silently runs the whole suite and passes
// for the wrong reason. Dropping it here keeps the documented verify honest.
import { spawn } from 'node:child_process';
const args = process.argv.slice(2).filter((a) => a !== '--');
spawn('vitest', ['run', ...args], { stdio: 'inherit', shell: false })
  .on('exit', (code) => process.exit(code ?? 1));
