import { createSignal, onCleanup, onMount, Show } from "solid-js";
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

  onMount(() => {
    let disposed = false;
    let cleanup = () => {};
    let managedSupervisor:
      | (HostLspSupervisor & { dispose?: () => Promise<void> })
      | undefined;
    onCleanup(() => {
      disposed = true;
      cleanup();
    });

    const setup = async () => {
      let supervisor = props.lsp;
      let client: ReturnType<typeof createLspClient> | null = null;

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
          if (!editorClient) return;
          const requestNumber = ++semanticRequest;
          try {
            editorClient.sync();
            const result = await requestSemanticTokens(
              editorClient,
              props.file.uri,
              semanticResult,
            );
            if (!result || disposed || requestNumber !== semanticRequest) return;
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
            : (languageExtensions(
                props.language,
                editorClient,
                props.file.uri,
              ) as Extension[])),
          ...(editorClient ? [semanticTokensExtension()] : []),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              props.onChange?.(update.state.doc.toString());
              queueSemanticRefresh();
            }
          }),
        ];
        const state = EditorState.create({
          doc: props.file.content,
          extensions: surfaceExtensions,
        });
        view = new EditorView({ state, parent: host });
        cleanup = () => {
          if (semanticTimer) clearTimeout(semanticTimer);
          view.destroy();
          editorClient?.disconnect();
          disposeManagedSupervisor();
        };
        if (editorClient) void editorClient.initializing.then(refreshSemanticTokens);
      };

      try {
        if (!supervisor && props.projectRoot && props.paneId) {
          managedSupervisor = await attachTauriLsp({
            projectRoot: props.projectRoot,
            projectId: props.projectId,
            paneId: props.paneId,
            filePath: props.file.path,
            onDiagnostics: props.onDiagnostics,
          });
          supervisor = managedSupervisor;
        }
        if (disposed) {
          disposeManagedSupervisor();
          return;
        }
        client = supervisor
          ? createLspClient(
              props.rootUri ?? "file:///workspace",
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
        if (disposed) {
          client?.disconnect();
          disposeManagedSupervisor();
          return;
        }
        mountEditor(client, false);
        setSurfaceState(props.file.content.length === 0 ? "empty" : "loaded");
      } catch (cause) {
        if (disposed) {
          client?.disconnect();
          disposeManagedSupervisor();
          return;
        }
        client?.disconnect();
        disposeManagedSupervisor();
        // Use the real file content, but remove all language/LSP extensions so
        // an unavailable server cannot leave an empty or unusable host.
        mountEditor(null, true);
        setError(failureMessage(cause));
        setSurfaceState("error");
      }
    };
    void setup();
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
