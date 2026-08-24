export { EditorSurface } from "./EditorSurface";
export type { EditorSurfaceProps } from "./EditorSurface";
export { EditorPane } from "./EditorPane";
export { FullWindowEditor } from "./FullWindowEditor";
export { MergeEditor, createMergeView } from "./MergeEditor";
export { editorKeymap, editorKeymapBindings } from "./keymap";
export { editorTheme, editorThemeTokens } from "./theme";
export {
  MultiFileWorkspace,
  createLspClient,
  languageExtensions,
  supervisorTransport,
} from "./lsp";
export type { HostLspSupervisor } from "./lsp";
export type { EditorFile, LanguageDescriptor } from "./types";
export { descriptorForPath, plainTextDescriptor } from "./types";
