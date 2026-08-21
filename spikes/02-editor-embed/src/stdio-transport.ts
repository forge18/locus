// An LSP Transport over a child process's stdio.
//
// @codemirror/lsp-client's Transport is deliberately dumb — send(string),
// subscribe(handler) — and knows nothing about framing. LSP on stdio uses
// Content-Length headers, so that belongs here.
//
// In Locus this code lives in the Rust host, not the webview: PLAN.md §Editor
// puts the server on the host under supervision, and §Two webviews sends
// high-frequency streams over tauri::ipc::Channel. This TypeScript version is
// what makes the SERVER side testable without a window — the question "does
// lsp-client drive a real rust-analyzer" is not a question about Tauri.
import type { ChildProcessWithoutNullStreams } from 'node:child_process';

export type Transport = {
  send(message: string): void;
  subscribe(handler: (value: string) => void): void;
  unsubscribe(handler: (value: string) => void): void;
};

export function stdioTransport(proc: ChildProcessWithoutNullStreams): Transport {
  const handlers = new Set<(value: string) => void>();
  let buffer = Buffer.alloc(0);

  proc.stdout.on('data', (chunk: Buffer) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const split = buffer.indexOf('\r\n\r\n');
      if (split < 0) return;
      const header = buffer.subarray(0, split).toString('ascii');
      const length = Number(/content-length:\s*(\d+)/i.exec(header)?.[1]);
      if (!Number.isFinite(length)) { buffer = buffer.subarray(split + 4); continue; }
      const start = split + 4;
      if (buffer.length < start + length) return;      // wait for the rest
      const body = buffer.subarray(start, start + length).toString('utf8');
      buffer = buffer.subarray(start + length);
      for (const h of [...handlers]) h(body);
    }
  });

  return {
    send(message) {
      const body = Buffer.from(message, 'utf8');
      proc.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
      proc.stdin.write(body);
    },
    subscribe(handler) { handlers.add(handler); },
    unsubscribe(handler) { handlers.delete(handler); },
  };
}
