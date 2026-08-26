import { completionKeymap } from "@codemirror/autocomplete";
import { defaultKeymap, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  findReferencesKeymap,
  formatKeymap,
  jumpToDefinitionKeymap,
  renameKeymap,
  signatureKeymap,
} from "@codemirror/lsp-client";
import { searchKeymap } from "@codemirror/search";
import { keymap } from "@codemirror/view";

/** The single keymap used by pane and full-window editors. */
export const editorKeymap = keymap.of([
  ...defaultKeymap,
  ...historyKeymap,
  ...completionKeymap,
  ...searchKeymap,
  ...formatKeymap,
  ...renameKeymap,
  ...jumpToDefinitionKeymap,
  ...findReferencesKeymap,
  ...signatureKeymap,
  indentWithTab,
]);

export const editorKeymapBindings = Object.freeze([
  ...defaultKeymap,
  ...historyKeymap,
  ...completionKeymap,
  ...searchKeymap,
  ...formatKeymap,
  ...renameKeymap,
  ...jumpToDefinitionKeymap,
  ...findReferencesKeymap,
  ...signatureKeymap,
  indentWithTab,
]);
