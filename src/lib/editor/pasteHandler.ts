/**
 * Paste Handler Extension for CodeMirror
 * Handles pasting images from clipboard, saves them to the vault,
 * and inserts an embed link at the cursor position.
 */

import { EditorView } from "@codemirror/view";
import { savePastedImage } from "../services/api";

/**
 * Get file extension from MIME type
 */
function getExtensionFromMime(mimeType: string): string {
  const mimeToExt: Record<string, string> = {
    "image/png": "png",
    "image/jpeg": "jpg",
    "image/gif": "gif",
    "image/webp": "webp",
    "image/svg+xml": "svg",
    "image/bmp": "bmp",
  };
  return mimeToExt[mimeType] || "png";
}

/**
 * Convert a Blob to base64 string
 */
async function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const base64 = reader.result as string;
      // Remove the data URL prefix (e.g., "data:image/png;base64,")
      const base64Data = base64.split(",")[1];
      resolve(base64Data);
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

/**
 * Handle paste event and check for images
 */
async function handlePaste(view: EditorView, event: ClipboardEvent): Promise<boolean> {
  const clipboardData = event.clipboardData;
  if (!clipboardData) return false;

  // Check for image items in clipboard
  const items = Array.from(clipboardData.items);
  const imageItem = items.find(item => item.type.startsWith("image/"));

  if (!imageItem) {
    // No image, let default paste handling occur
    return false;
  }

  // Prevent default paste behavior
  event.preventDefault();

  const blob = imageItem.getAsFile();
  if (!blob) {
    console.error("[PasteHandler] Failed to get image blob from clipboard");
    return true;
  }

  try {
    // Convert to base64
    const base64Data = await blobToBase64(blob);
    const extension = getExtensionFromMime(imageItem.type);

    // Save the image via Tauri command
    const savedPath = await savePastedImage(base64Data, extension);
    console.log("[PasteHandler] Saved image to:", savedPath);

    // Insert embed link at cursor position
    const embedText = `![[${savedPath}]]`;
    const { from, to } = view.state.selection.main;

    view.dispatch({
      changes: { from, to, insert: embedText },
      selection: { anchor: from + embedText.length },
    });

    return true;
  } catch (error) {
    console.error("[PasteHandler] Failed to save pasted image:", error);
    return true;
  }
}

/**
 * Extension that provides image paste handling
 */
export function pasteHandlerExtension() {
  return EditorView.domEventHandlers({
    paste(event: ClipboardEvent, view: EditorView) {
      // Handle async paste - we return false to not block, but handle it ourselves
      handlePaste(view, event);

      // Check if there's an image in clipboard to decide whether to prevent default
      const clipboardData = event.clipboardData;
      if (clipboardData) {
        const items = Array.from(clipboardData.items);
        const hasImage = items.some(item => item.type.startsWith("image/"));
        if (hasImage) {
          // Prevent default if we have an image
          return true;
        }
      }

      // Let default handling occur for non-image pastes
      return false;
    },
  });
}
