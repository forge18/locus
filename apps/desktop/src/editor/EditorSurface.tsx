import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { history } from "@codemirror/commands";
import {
  lineNumbers,
  highlightActiveLine,
  drawSelection,
  EditorView,
} from "@codemirror/view";
import { EditorState, type Extension } from "@codemirror/state";
import type { HostLspSupervisor, LspDiagnostics } from "./lsp";
import { createLspClient, languageExtensions } from "./lsp";
import { attachTauriLsp } from "./tauriLsp";
import {
  applySemanticTokens,
  decodeSemanticTokens,
  requestSemanticTokens,
  semanticTokensExtension,
  type SemanticTokenResult,
} from "./semanticTokens";
import { editorKeymap } from "./keymap";
import { editorTheme } from "./theme";
import { InlineError } from "../ui/InlineError";
import type { EditorFile, LanguageDescriptor } from "./types";

export type EditorSurfaceState = "loading" | "empty" | "error" | "loaded";

function failureMessage(cause: unknown): string {
  if (cause instanceof Error && cause.message) return cause.message;
  if (typeof cause === "string" && cause) return cause;
  return "The language service could not be started.";
}

export interface EditorSurfaceProps {
  file: EditorFile;
  language: LanguageDescriptor;
  lsp?: HostLspSupervisor;
  rootUri?: string;
  projectRoot?: string;
  projectId?: string;
  paneId?: string;
  onChange?: (content: string) => void;
  onDiagnostics?: (diagnostics: LspDiagnostics) => void;
}

/** The one CodeMirror implementation shared by the pane and full-window module. */
export function EditorSurface(props: EditorSurfaceProps) {
  let host!: HTMLDivElement;
  const [surfaceState, setSurfaceState] =
    createSignal<EditorSurfaceState>("loading");
  const [error, setError] = createSignal<string>();

  let disposed = false;
  let activeConfig: string | undefined;
  let setupGeneration = 0;
  let cleanup = () => {};
  let activeView: EditorView | undefined;
  let activeFallback = false;

  const syncFileContent = (file: EditorFile) => {
    const view = activeView;
    if (!view) return;
    if (view.state.doc.toString() !== file.content) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: file.content,
        },
      });
    }
    if (!activeFallback) {
      setSurfaceState(file.content.length === 0 ? "empty" : "loaded");
    }
  };

  const setupFile = async (
    file: EditorFile,
    language: LanguageDescriptor,
    supervisor: HostLspSupervisor | undefined,
    rootUri: string,
    projectRoot: string | undefined,
    projectId: string | undefined,
    paneId: string | undefined,
    generation: number,
  ) => {
    let managedSupervisor:
      | (HostLspSupervisor & { dispose?: () => Promise<void> })
      | undefined;
    let client: ReturnType<typeof createLspClient> | null = null;
    const isCurrent = () => !disposed && generation === setupGeneration;

    const disposeManagedSupervisor = () => {
      const pendingDispose = managedSupervisor?.dispose?.();
      managedSupervisor = undefined;
      void pendingDispose?.catch(() => undefined);
    };

    const mountEditor = (editorClient: typeof client, plainText: boolean) => {
      let view!: EditorView;
      let semanticResult: SemanticTokenResult | undefined;
      let semanticRequest = 0;
      let semanticTimer: ReturnType<typeof setTimeout> | undefined;
      const refreshSemanticTokens = async () => {
        if (!editorClient || !isCurrent()) return;
        const requestNumber = ++semanticRequest;
        try {
          editorClient.sync();
          const result = await requestSemanticTokens(
            editorClient,
            file.uri,
            semanticResult,
          );
          if (!result || !isCurrent() || requestNumber !== semanticRequest) {
            return;
          }
          semanticResult = result;
          applySemanticTokens(view, decodeSemanticTokens(result.data));
        } catch {
          // Servers without semantic-token support degrade to ordinary editable text.
        }
      };
      const queueSemanticRefresh = () => {
        if (semanticTimer) clearTimeout(semanticTimer);
        semanticTimer = setTimeout(() => void refreshSemanticTokens(), 150);
      };
      const surfaceExtensions: Extension[] = [
        editorKeymap,
        history(),
        lineNumbers(),
        drawSelection(),
        highlightActiveLine(),
        editorTheme,
        // A failed LSP setup must not prevent the file from being edited.
        ...(plainText
          ? []
          : (languageExtensions(language, editorClient, file.uri) as Extension[])),
        ...(editorClient ? [semanticTokensExtension()] : []),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            props.onChange?.(update.state.doc.toString());
            queueSemanticRefresh();
          }
        }),
      ];
      const state = EditorState.create({
        doc: file.content,
        extensions: surfaceExtensions,
      });
      view = new EditorView({ state, parent: host });
      const dispose = () => {
        if (semanticTimer) clearTimeout(semanticTimer);
        view.destroy();
        editorClient?.disconnect();
        disposeManagedSupervisor();
      };
      if (!isCurrent()) {
        dispose();
        return;
      }
      activeView = view;
      cleanup = () => {
        dispose();
        if (activeView === view) activeView = undefined;
      };
      if (editorClient) void editorClient.initializing.then(refreshSemanticTokens);
    };

    try {
      if (!supervisor && projectRoot && paneId) {
        managedSupervisor = await attachTauriLsp({
          projectRoot,
          projectId,
          paneId,
          filePath: file.path,
          onDiagnostics: props.onDiagnostics,
        });
        supervisor = managedSupervisor;
      }
      if (!isCurrent()) {
        disposeManagedSupervisor();
        return;
      }
      client = supervisor
        ? createLspClient(
            rootUri,
            {
              send: (message) => supervisor!.send(message),
              subscribe: (handler) => supervisor!.subscribe(handler),
              unsubscribe: (handler) => supervisor!.unsubscribe(handler),
            },
            { onDiagnostics: props.onDiagnostics },
          )
        : null;
      // Keep the loading state visible until the host confirms that its LSP
      // connection is ready. A rejected setup takes the plain-text path below.
      if (client) await client.initializing;
      if (!isCurrent()) {
        client?.disconnect();
        disposeManagedSupervisor();
        return;
      }
      mountEditor(client, false);
      const currentFile = props.file;
      syncFileContent(currentFile);
      setError(undefined);
      setSurfaceState(
        currentFile.content.length === 0 ? "empty" : "loaded",
      );
    } catch (cause) {
      if (!isCurrent()) {
        client?.disconnect();
        disposeManagedSupervisor();
        return;
      }
      client?.disconnect();
      disposeManagedSupervisor();
      // Use the real file content, but remove all language/LSP extensions so
      // an unavailable server cannot leave an empty or unusable host.
      activeFallback = true;
      mountEditor(null, true);
      syncFileContent(props.file);
      setError(failureMessage(cause));
      setSurfaceState("error");
    }
  };

  createEffect(() => {
    const file = props.file;
    const language = props.language;
    const rootUri = props.rootUri ?? "file:///workspace";
    const projectRoot = props.projectRoot;
    const projectId = props.projectId;
    const paneId = props.paneId;
    const config = [
      file.uri,
      file.path,
      file.languageId,
      language.id,
      language.grammar ?? "",
      language.extensions.join(","),
      rootUri,
      projectRoot ?? "",
      projectId ?? "",
      paneId ?? "",
      props.lsp ? "provided" : "managed",
    ].join("\u0000");
    if (activeConfig === config) {
      syncFileContent(file);
      return;
    }

    activeConfig = config;
    setupGeneration += 1;
    cleanup();
    cleanup = () => {};
    activeView = undefined;
    activeFallback = false;
    setError(undefined);
    setSurfaceState("loading");
    void setupFile(
      { ...file },
      language,
      props.lsp,
      rootUri,
      projectRoot,
      projectId,
      paneId,
      setupGeneration,
    );
  });

  onCleanup(() => {
    disposed = true;
    setupGeneration += 1;
    cleanup();
    cleanup = () => {};
    activeView = undefined;
  });
  return (
    <div
      class="locus-editor-surface"
      data-testid="editor-surface"
      data-state={surfaceState()}
      data-editor-state={surfaceState()}
      ref={host}
    >
      <Show when={surfaceState() === "loading"}>
        <div class="editor-state" data-testid="editor-loading" role="status">
          Loading editor…
        </div>
      </Show>
      <Show when={surfaceState() === "empty"}>
        <div class="editor-state" data-testid="editor-empty" role="status">
          Empty file
        </div>
      </Show>
      <Show when={surfaceState() === "error"}>
        <div class="editor-state" data-testid="editor-error">
          <InlineError
            cause={error() ?? "The language service could not be started."}
            next="Editing continues in plain text."
          />
        </div>
      </Show>
    </div>
  );
}

export default EditorSurface;
