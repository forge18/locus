import EditorSurface, { type EditorSurfaceProps } from "./EditorSurface";
import "./editor.css";

/** Side-pane zoom of the shared editor surface. */
export function EditorPane(props: EditorSurfaceProps) {
  return (
    <section class="locus-editor-pane" data-testid="editor-pane" data-zoom="pane">
      <EditorSurface {...props} />
    </section>
  );
}

export default EditorPane;
