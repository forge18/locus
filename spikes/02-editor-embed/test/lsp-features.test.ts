import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { resolve } from 'node:path';
import { startSession, positionOf, type Session } from '../src/lsp-session';

// Q1: all five features the editor needs, answered PER FEATURE.
// "LSP works" is not a verdict; four working and one missing is a different M2.
const ROOT = resolve(process.cwd(), 'fixture');
let s: Session;

beforeAll(async () => { s = await startSession(ROOT); }, 120_000);
afterAll(() => s?.stop());

describe('the five features, against a real rust-analyzer', () => {
  it('completion returns real symbols from the crate', async () => {
    const at = positionOf(s.text, 'total(&values)');
    const res: any = await s.client.request('textDocument/completion', {
      textDocument: { uri: s.uri }, position: { line: at.line, character: at.character + 3 },
    });
    const items = res?.items ?? res ?? [];
    expect(items.length).toBeGreaterThan(0);
    expect(items.map((i: any) => i.label)).toContain('total');
  });

  it('hover returns the doc comment, not just a type', async () => {
    const at = positionOf(s.text, 'pub fn total');
    const res: any = await s.client.request('textDocument/hover', {
      textDocument: { uri: s.uri }, position: { line: at.line, character: at.character + 8 },
    });
    const value = res?.contents?.value ?? '';
    expect(value).toContain('fn total');
    expect(value, 'the doc comment should come through').toContain('Sums a slice');
  });

  it('go to definition resolves a call site to its declaration', async () => {
    const at = positionOf(s.text, 'total(values) /');
    const res: any = await s.client.request('textDocument/definition', {
      textDocument: { uri: s.uri }, position: { line: at.line, character: at.character + 2 },
    });
    const target = Array.isArray(res) ? res[0] : res;
    expect(target, 'no definition returned').toBeTruthy();
    const declared = positionOf(s.text, 'pub fn total');
    expect((target.range ?? target.targetSelectionRange).start.line).toBe(declared.line);
  });

  it('find references returns BOTH call sites, not only the nearest', async () => {
    const at = positionOf(s.text, 'pub fn total');
    const res: any = await s.client.request('textDocument/references', {
      textDocument: { uri: s.uri }, position: { line: at.line, character: at.character + 8 },
      context: { includeDeclaration: false },
    });
    expect(res.length).toBeGreaterThanOrEqual(2);
  });

  it('diagnostics arrive PUSHED, carrying a real message and a range', () => {
    // Not a capability flag — the server sends these unprompted, which is why
    // the session registers a notification handler rather than requesting them.
    expect(s.diagnostics.length, 'no diagnostics were pushed').toBeGreaterThan(0);
    const d: any = s.diagnostics[0];
    expect(d.message).toBeTruthy();
    expect(d.range.start.line).toBeGreaterThan(0);
    console.log(JSON.stringify({ count: s.diagnostics.length, first: d.message }));
  });
});
