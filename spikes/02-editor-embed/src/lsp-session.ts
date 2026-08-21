// Starting a real rust-analyzer and getting to the point where it will answer.
//
// Shared by the tests because the interesting part is not the plumbing: it is
// that rust-analyzer does not answer ANYTHING until it has indexed, and it
// announces that through $/progress rather than through the initialize
// response. A client that asks too early gets empty results, not an error —
// which is exactly the shape of bug that would read as "LSP does not work".
import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { LSPClient } from '@codemirror/lsp-client';
import { stdioTransport } from './stdio-transport';
import { findRustAnalyzer } from './find-rust-analyzer';

export type Session = {
  client: LSPClient;
  proc: ChildProcessWithoutNullStreams;
  uri: string;
  text: string;
  diagnostics: unknown[];
  stop(): void;
};

export async function startSession(root: string): Promise<Session> {
  const file = join(root, 'src/main.rs');
  const uri = pathToFileURL(file).href;
  const text = readFileSync(file, 'utf8');

  const proc = spawn(findRustAnalyzer(), [], { cwd: root, stdio: 'pipe' });
  proc.stderr.resume();

  const diagnostics: unknown[] = [];
  let indexed: () => void;
  const indexing = new Promise<void>((r) => { indexed = r; });

  const client = new LSPClient({
    rootUri: pathToFileURL(root).href,
    timeout: 60_000,
    notificationHandlers: {
      // Diagnostics are PUSHED, which is why they are not a capability flag.
      'textDocument/publishDiagnostics': (_c, params: any) => {
        if (params.uri === uri && params.diagnostics?.length) diagnostics.push(...params.diagnostics);
        return true;
      },
      '$/progress': (_c, params: any) => {
        if (params.value?.kind === 'end') indexed();
        return true;
      },
    },
  });
  client.connect(stdioTransport(proc));
  await client.initializing;

  client.notification('textDocument/didOpen', {
    textDocument: { uri, languageId: 'rust', version: 1, text },
  });

  // Bounded: if rust-analyzer never reports done, the tests should fail on the
  // assertion rather than hang on the wait.
  await Promise.race([indexing, new Promise((r) => setTimeout(r, 90_000))]);

  return { client, proc, uri, text, diagnostics, stop: () => { client.disconnect(); proc.kill(); } };
}

/** Zero-based line/character of the first occurrence of `needle`. */
export function positionOf(text: string, needle: string, occurrence = 1): { line: number; character: number } {
  let index = -1;
  for (let i = 0; i < occurrence; i++) index = text.indexOf(needle, index + 1);
  const before = text.slice(0, index);
  const line = before.split('\n').length - 1;
  return { line, character: index - (before.lastIndexOf('\n') + 1) };
}
