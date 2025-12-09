/**
 * CodeMirror editor extensions and configuration
 */

// Shared editor configuration
export {
  editorTheme,
  createSaveKeymap,
  createEditorExtensions,
  type EditorExtensionOptions,
} from "./editorConfig";

// Individual extensions (for advanced use cases)
export { wikiLinkCompletion, invalidateNotesCache } from "./wikiLinkCompletion";
export { livePreview } from "./livePreview";
export { markdownHighlight } from "./markdownHighlight";
export { embedExtension, invalidateEmbedCache } from "./embedExtension";
export { linkHandlerExtension, isPositionInWikiLink } from "./linkHandler";
export { hoverPreviewExtension, invalidatePreviewCache } from "./hoverPreview";
export { pasteHandlerExtension } from "./pasteHandler";
export { vimExtension, isVimModeEnabled } from "./vimExtension";
export { calloutExtension } from "./calloutExtension";
