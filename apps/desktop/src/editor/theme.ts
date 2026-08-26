import { EditorView } from "@codemirror/view";

/** One semantic CodeMirror theme for both editor zoom levels. */
export const editorTheme = EditorView.theme(
  {
    "&": {
      color: "var(--text-primary)",
      backgroundColor: "var(--surface-ground)",
      height: "100%",
    },
    ".cm-content": {
      caretColor: "var(--action-attention)",
      fontFamily: "var(--fm)",
      fontSize: "var(--t-body)",
      padding: "12px 0",
    },
    ".cm-gutters": {
      backgroundColor: "var(--surface-chrome)",
      color: "var(--text-muted)",
      border: "0",
      borderRight: "1px solid var(--border-subtle)",
    },
    ".cm-activeLine, .cm-activeLineGutter": {
      backgroundColor: "var(--surface-raised)",
    },
    ".cm-selectionBackground, ::selection": {
      backgroundColor: "var(--surface-selected) !important",
    },
    ".cm-tooltip": {
      backgroundColor: "var(--surface-elevated)",
      color: "var(--text-primary)",
      border: "1px solid var(--border-strong)",
    },
  },
  { dark: true },
);

export const editorThemeTokens = Object.freeze([
  "--surface-ground",
  "--surface-chrome",
  "--surface-raised",
  "--surface-selected",
  "--surface-elevated",
  "--text-primary",
  "--text-muted",
  "--action-attention",
  "--border-subtle",
  "--border-strong",
] as const);
