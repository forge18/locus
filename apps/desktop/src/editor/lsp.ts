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
import { javascriptLanguage, typescriptLanguage } from "@codemirror/lang-javascript";
import { ChangeSet, EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import type { LanguageDescriptor } from "./types";

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

type OpenFile = WorkspaceFile & { view: EditorView | null };

/** Tracks all open files for one project rather than limiting LSP to one editor. */
export class MultiFileWorkspace extends Workspace {
  files: OpenFile[] = [];

  openFile(uri: string, languageId: string, view: EditorView): void {
    const current = this.files.find((file) => file.uri === uri);
    if (current) {
      current.view = view;
      return;
    }
    const file: OpenFile = {
      uri,
      languageId,
      version: 1,
      doc: view.state.doc,
      view,
      getView: () => file.view,
    };
    this.files = [...this.files, file];
    if (this.client.connected) this.client.didOpen(file);
  }

  closeFile(uri: string, _view: EditorView): void {
    const file = this.files.find((candidate) => candidate.uri === uri);
    if (file && this.client.connected) this.client.didClose(uri);
    this.files = this.files.filter((candidate) => candidate.uri !== uri);
  }

  syncFiles() {
    const updates = [];
    for (const file of this.files) {
      if (!file.view || file.view.state.doc.eq(file.doc)) continue;
      const previous = file.doc;
      const changes = ChangeSet.of(
        [{ from: 0, to: previous.length, insert: file.view.state.doc.toString() }],
        previous.length,
      );
      file.doc = file.view.state.doc;
      file.version += 1;
      updates.push({ file, prevDoc: previous, changes });
    }
    return updates;
  }

  displayFile(uri: string): Promise<EditorView | null> {
    return Promise.resolve(this.files.find((file) => file.uri === uri)?.view ?? null);
  }
}

export function createLspClient(rootUri: string, transport?: Transport): LSPClient {
  const client = new LSPClient({
    rootUri,
    extensions: languageServerExtensions(),
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
