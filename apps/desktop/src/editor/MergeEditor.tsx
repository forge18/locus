import { onCleanup, onMount } from "solid-js";
import { history } from "@codemirror/commands";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { MergeView } from "@codemirror/merge";
import { editorKeymap } from "./keymap";
import { editorTheme } from "./theme";
import "./editor.css";

export interface MergeEditorProps {
  base: string;
  agent: string;
  onBaseChange?: (content: string) => void;
}

export function createMergeView(
  parent: Element,
  base: string,
  agent: string,
): MergeView {
  return new MergeView({
    parent,
    orientation: "a-b",
    // A is the base clone; reverting a chunk copies the agent change into it.
    revertControls: "b-to-a",
    collapseUnchanged: { margin: 3, minSize: 4 },
    gutter: true,
    a: {
      doc: base,
      extensions: [editorKeymap, history(), editorTheme],
    },
    b: {
      doc: agent,
      extensions: [
        editorKeymap,
        editorTheme,
        EditorState.readOnly.of(true),
        EditorView.editable.of(false),
      ],
    },
    renderRevertControl: () => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "locus-merge-revert";
      button.title = "Revert this chunk into the base";
      button.textContent = "←";
      return button;
    },
  });
}

export function MergeEditor(props: MergeEditorProps) {
  let host!: HTMLDivElement;
  onMount(() => {
    const merge = createMergeView(host, props.base, props.agent);
    const listener = EditorView.updateListener.of((update) => {
      if (update.docChanged) props.onBaseChange?.(update.state.doc.toString());
    });
    // The base editor is the writable side. Reconfigure it with the listener without
    // replacing the shared theme or keymap.
    merge.a.dispatch({ effects: StateEffect.appendConfig.of(listener) });
    onCleanup(() => merge.destroy());
  });
  return <div class="locus-merge-editor" data-testid="merge-view" ref={host} />;
}

// Kept local to avoid a second merge-editor configuration path.
import { StateEffect } from "@codemirror/state";

export default MergeEditor;
