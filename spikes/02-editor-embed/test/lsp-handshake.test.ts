import { afterAll, describe, expect, it } from 'vitest';
import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { LSPClient } from '@codemirror/lsp-client';
import { pathToFileURL } from 'node:url';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { stdioTransport } from '../src/stdio-transport';
import { findRustAnalyzer } from '../src/find-rust-analyzer';

// Q1, first half: does @codemirror/lsp-client complete an initialize exchange
// with a REAL rust-analyzer? Not a mock, not a fixture recording — the binary.
// NOTE: `new URL('../fixture/', import.meta.url)` does NOT work here — vitest
// rewrites import.meta.url and it resolves to '/fixture'. cwd is the package
// root under vitest, so resolve from there.
const ROOT = resolve(process.cwd(), 'fixture');

let proc: ChildProcessWithoutNullStreams;
afterAll(() => proc?.kill());

describe('a real rust-analyzer over @codemirror/lsp-client', () => {
  it('initializes and advertises the capabilities the editor needs', async () => {
    const bin = findRustAnalyzer();
    expect(existsSync(ROOT), `fixture crate missing at ${ROOT}`).toBe(true);
    proc = spawn(bin, [], { cwd: ROOT, stdio: 'pipe' });
    const client = new LSPClient({ rootUri: pathToFileURL(ROOT).href, timeout: 60_000 });
    client.connect(stdioTransport(proc));
    await client.initializing;

    const caps = client.serverCapabilities!;
    expect(caps, 'no capabilities came back').toBeTruthy();

    // Answered per feature, not in aggregate: four working and one missing is a
    // different M2 than five working.
    expect(caps.completionProvider, 'completion').toBeTruthy();
    expect(caps.hoverProvider, 'hover').toBeTruthy();
    expect(caps.definitionProvider, 'go to definition').toBeTruthy();
    expect(caps.referencesProvider, 'find references').toBeTruthy();
    // Diagnostics are pushed, not requested, so they are not a capability flag.
    // lsp-features.test.ts is where they are actually observed arriving.

    // Not required by PLAN.md, but recorded because they are cheap wins the
    // editor gets for free if present.
    console.log(JSON.stringify({
      rename: Boolean(caps.renameProvider),
      formatting: Boolean(caps.documentFormattingProvider),
      signatureHelp: Boolean(caps.signatureHelpProvider),
      semanticTokens: Boolean(caps.semanticTokensProvider),
      inlayHints: Boolean(caps.inlayHintProvider),
    }));
  });
});
