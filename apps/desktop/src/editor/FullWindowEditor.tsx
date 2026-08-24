import EditorSurface, { type EditorSurfaceProps } from "./EditorSurface";
import "./editor.css";

/** Full-window zoom of the exact same shared editor surface. */
export function FullWindowEditor(props: EditorSurfaceProps) {
  return (
    <main class="locus-editor-full-window" data-testid="full-window-editor" data-zoom="full">
      <EditorSurface {...props} />
    </main>
  );
}

export default FullWindowEditor;
