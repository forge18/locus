import { onCleanup, onMount } from "solid-js";
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
import type { EditorFile, LanguageDescriptor } from "./types";

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
        await managedSupervisor?.dispose?.();
        return;
      }
      const client = supervisor
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
      let view!: EditorView;
      let semanticResult: SemanticTokenResult | undefined;
      let semanticRequest = 0;
      let semanticTimer: ReturnType<typeof setTimeout> | undefined;
      const refreshSemanticTokens = async () => {
        if (!client) return;
        const requestNumber = ++semanticRequest;
        try {
          client.sync();
          const result = await requestSemanticTokens(
            client,
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
        ...(languageExtensions(
          props.language,
          client,
          props.file.uri,
        ) as Extension[]),
        ...(client ? [semanticTokensExtension()] : []),
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
      if (client)
        void client.initializing
          .then(refreshSemanticTokens)
          .catch(() => undefined);
      cleanup = () => {
        if (semanticTimer) clearTimeout(semanticTimer);
        view.destroy();
        client?.disconnect();
        const pendingDispose = managedSupervisor?.dispose?.();
        void pendingDispose?.catch(() => undefined);
      };
    };
    void setup().catch(() => {
      const pendingDispose = managedSupervisor?.dispose?.();
      void pendingDispose?.catch(() => undefined);
    });
  });
  return (
    <div class="locus-editor-surface" data-testid="editor-surface" ref={host} />
  );
}

export default EditorSurface;
