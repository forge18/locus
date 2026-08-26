import {
  LSPClient,
  Workspace,
  type Transport,
  type WorkspaceFile,
  hoverTooltips,
  languageServerExtensions,
  serverCompletion,
  signatureHelp,
} from "@codemirror/lsp-client";
import {
  javascriptLanguage,
  typescriptLanguage,
} from "@codemirror/lang-javascript";
import { ChangeSet, EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import type { LanguageDescriptor } from "./types";

export interface LspDiagnostics {
  uri: string;
  diagnostics: readonly unknown[];
  version?: number;
}

export interface LspClientOptions {
  onDiagnostics?: (diagnostics: LspDiagnostics) => void;
}

/** The host-side LSP supervisor bridge. It deliberately knows no harness. */
export interface HostLspSupervisor {
  send(message: string): void;
  subscribe(handler: (message: string) => void): void;
  unsubscribe(handler: (message: string) => void): void;
}

export function supervisorTransport(supervisor: HostLspSupervisor): Transport {
  return {
    send: (message) => supervisor.send(message),
    subscribe: (handler) => supervisor.subscribe(handler),
    unsubscribe: (handler) => supervisor.unsubscribe(handler),
  };
}

type OpenFile = WorkspaceFile & {
  view: EditorView | null;
  views: Set<EditorView>;
};

// Multiple panes can own clients connected to the same project server. Keep the server-side
// document open until the last pane releases the URI instead of sending an early didClose.
const documentReferences = new Map<string, number>();

/** Tracks all open files for one project rather than limiting LSP to one editor. */
export class MultiFileWorkspace extends Workspace {
  files: OpenFile[] = [];

  openFile(uri: string, languageId: string, view: EditorView): void {
    const current = this.files.find((file) => file.uri === uri);
    if (current) {
      current.views.add(view);
      current.view = view;
      return;
    }
    const file: OpenFile = {
      uri,
      languageId,
      version: 1,
      doc: view.state.doc,
      view,
      views: new Set([view]),
      getView: () => file.view,
    };
    this.files = [...this.files, file];
    const references = documentReferences.get(uri) ?? 0;
    documentReferences.set(uri, references + 1);
    if (this.client.connected && references === 0) this.client.didOpen(file);
  }

  closeFile(uri: string, view: EditorView): void {
    const file = this.files.find((candidate) => candidate.uri === uri);
    if (!file) return;
    file.views.delete(view);
    if (file.views.size > 0) {
      file.view = file.views.values().next().value ?? null;
      return;
    }
    const references = documentReferences.get(uri) ?? 1;
    if (references <= 1) {
      documentReferences.delete(uri);
      if (this.client.connected) this.client.didClose(uri);
    } else {
      documentReferences.set(uri, references - 1);
    }
    this.files = this.files.filter((candidate) => candidate.uri !== uri);
  }

  syncFiles() {
    const updates = [];
    for (const file of this.files) {
      if (!file.view || file.view.state.doc.eq(file.doc)) continue;
      const previous = file.doc;
      const changes = ChangeSet.of(
        [
          {
            from: 0,
            to: previous.length,
            insert: file.view.state.doc.toString(),
          },
        ],
        previous.length,
      );
      file.doc = file.view.state.doc;
      file.version += 1;
      updates.push({ file, prevDoc: previous, changes });
    }
    return updates;
  }

  displayFile(uri: string): Promise<EditorView | null> {
    return Promise.resolve(
      this.files.find((file) => file.uri === uri)?.view ?? null,
    );
  }
}

export function createLspClient(
  rootUri: string,
  transport?: Transport,
  options: LspClientOptions = {},
): LSPClient {
  const client = new LSPClient({
    rootUri,
    extensions: [
      ...languageServerExtensions(),
      {
        clientCapabilities: {
          textDocument: {
            semanticTokens: {
              dynamicRegistration: false,
              requests: { full: { delta: true } },
            },
          },
        },
      },
    ],
    notificationHandlers: options.onDiagnostics
      ? {
          "textDocument/publishDiagnostics": (_client, params) => {
            options.onDiagnostics?.(params as LspDiagnostics);
            return false;
          },
        }
      : undefined,
    workspace: (connected) => new MultiFileWorkspace(connected),
  });
  if (transport) client.connect(transport);
  return client;
}

export function languageExtensions(
  descriptor: LanguageDescriptor,
  client: LSPClient | null,
  uri: string,
): readonly Extension[] {
  const extensions: Extension[] = [];
  if (descriptor.grammar === "javascript") extensions.push(javascriptLanguage);
  if (descriptor.grammar === "typescript") extensions.push(typescriptLanguage);
  if (client) {
    extensions.push(
      client.plugin(uri, descriptor.id),
      serverCompletion(),
      hoverTooltips(),
      signatureHelp(),
    );
  }
  return extensions;
}

export function plainEditorState(content: string): EditorState {
  return EditorState.create({ doc: content });
}
