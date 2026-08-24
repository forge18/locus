import { onCleanup, onMount } from "solid-js";
import { history } from "@codemirror/commands";
import { lineNumbers, highlightActiveLine, drawSelection, EditorView } from "@codemirror/view";
import { EditorState, type Extension } from "@codemirror/state";
import type { HostLspSupervisor } from "./lsp";
import { createLspClient, languageExtensions } from "./lsp";
import { editorKeymap } from "./keymap";
import { editorTheme } from "./theme";
import type { EditorFile, LanguageDescriptor } from "./types";

export interface EditorSurfaceProps {
  file: EditorFile;
  language: LanguageDescriptor;
  lsp?: HostLspSupervisor;
  rootUri?: string;
  onChange?: (content: string) => void;
}

/** The one CodeMirror implementation shared by the pane and full-window module. */
export function EditorSurface(props: EditorSurfaceProps) {
  let host!: HTMLDivElement;
  onMount(() => {
    const client = props.lsp
      ? createLspClient(props.rootUri ?? "file:///workspace", {
          send: (message) => props.lsp!.send(message),
          subscribe: (handler) => props.lsp!.subscribe(handler),
          unsubscribe: (handler) => props.lsp!.unsubscribe(handler),
        })
      : null;
    const surfaceExtensions: Extension[] = [
      editorKeymap,
      history(),
      lineNumbers(),
      drawSelection(),
      highlightActiveLine(),
      editorTheme,
      ...(languageExtensions(props.language, client, props.file.uri) as Extension[]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) props.onChange?.(update.state.doc.toString());
      }),
    ];
    const state = EditorState.create({
      doc: props.file.content,
      extensions: surfaceExtensions,
    });
    const view = new EditorView({ state, parent: host });
    onCleanup(() => {
      view.destroy();
      client?.disconnect();
    });
  });
  return <div class="locus-editor-surface" data-testid="editor-surface" ref={host} />;
}

export default EditorSurface;
