import { Decoration, type DecorationSet, EditorView } from "@codemirror/view";
import {
  type EditorState,
  RangeSetBuilder,
  StateEffect,
  StateField,
  type Extension,
} from "@codemirror/state";
import type { LSPClient } from "@codemirror/lsp-client";

export interface SemanticToken {
  line: number;
  start: number;
  length: number;
  tokenType: number;
  modifiers: number;
}

export interface SemanticTokenDeltaEdit {
  start: number;
  deleteCount: number;
  data: number[];
}

const setSemanticTokens = StateEffect.define<readonly SemanticToken[]>();

function tokenClass(token: SemanticToken): string {
  return `cm-lsp-semantic-token-${token.tokenType}`;
}

function decorationsFor(
  state: EditorState,
  tokens: readonly SemanticToken[],
): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const ranges = tokens
    .map((token) => {
      if (token.length <= 0 || token.line < 0 || token.start < 0) return null;
      if (token.line >= state.doc.lines) return null;
      const line = state.doc.line(token.line + 1);
      const from = line.from + Math.min(token.start, line.length);
      const to = Math.min(from + token.length, line.to);
      return to > from ? { from, to, token } : null;
    })
    .filter(
      (range): range is { from: number; to: number; token: SemanticToken } =>
        range !== null,
    )
    .sort((left, right) => left.from - right.from || left.to - right.to);
  for (const range of ranges) {
    builder.add(
      range.from,
      range.to,
      Decoration.mark({ class: tokenClass(range.token) }),
    );
  }
  return builder.finish();
}

export const semanticTokenField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    let next = value.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(setSemanticTokens))
        next = decorationsFor(transaction.state, effect.value);
    }
    return next;
  },
  provide: (field) => EditorView.decorations.from(field),
});

export function semanticTokensExtension(): Extension {
  return semanticTokenField;
}

export function applySemanticTokens(
  view: EditorView,
  tokens: readonly SemanticToken[],
): void {
  view.dispatch({ effects: setSemanticTokens.of(tokens) });
}

export function decodeSemanticTokens(data: readonly number[]): SemanticToken[] {
  if (data.length % 5 !== 0)
    throw new Error("semantic token data must contain five integers per token");
  let line = 0;
  let start = 0;
  const tokens: SemanticToken[] = [];
  for (let index = 0; index < data.length; index += 5) {
    const lineDelta = data[index];
    line += lineDelta;
    start = lineDelta === 0 ? start + data[index + 1] : data[index + 1];
    tokens.push({
      line,
      start,
      length: data[index + 2],
      tokenType: data[index + 3],
      modifiers: data[index + 4],
    });
  }
  return tokens;
}

export function applySemanticTokenDelta(
  previous: readonly number[],
  edits: readonly SemanticTokenDeltaEdit[],
): number[] {
  const next = [...previous];
  for (const edit of [...edits].sort(
    (left, right) => right.start - left.start,
  )) {
    if (
      edit.start < 0 ||
      edit.deleteCount < 0 ||
      edit.start + edit.deleteCount > next.length
    ) {
      throw new Error("semantic token delta is out of bounds");
    }
    if (edit.data.length % 5 !== 0)
      throw new Error("semantic token delta is not token aligned");
    next.splice(edit.start, edit.deleteCount, ...edit.data);
  }
  return next;
}

export interface SemanticTokenResult {
  data: number[];
  resultId?: string;
}

export interface SemanticTokenDeltaResult {
  edits: SemanticTokenDeltaEdit[];
  resultId?: string;
}

export async function requestSemanticTokens(
  client: LSPClient,
  uri: string,
  previous?: SemanticTokenResult,
): Promise<SemanticTokenResult | null> {
  const provider = client.serverCapabilities?.semanticTokensProvider;
  if (!provider || typeof provider === "boolean") return null;
  const full = provider.full;
  const supportsFull = full === true || typeof full === "object";
  if (!supportsFull) return null;
  const supportsDelta = typeof full === "object" && full.delta === true;
  if (previous?.resultId && supportsDelta) {
    try {
      const delta = await client.request<
        { textDocument: { uri: string }; previousResultId: string },
        SemanticTokenDeltaResult | SemanticTokenResult
      >("textDocument/semanticTokens/full/delta", {
        textDocument: { uri },
        previousResultId: previous.resultId,
      });
      if ("edits" in delta) {
        return {
          data: applySemanticTokenDelta(previous.data, delta.edits),
          resultId: delta.resultId,
        };
      }
      return delta;
    } catch {
      // A rejected delta is recoverable: request a complete result below.
    }
  }
  return client.request<{ textDocument: { uri: string } }, SemanticTokenResult>(
    "textDocument/semanticTokens/full",
    { textDocument: { uri } },
  );
}
