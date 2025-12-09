<script lang="ts">
  /**
   * ColumnEditor - A self-contained editor for a single document in doc-finder mode.
   * Each column gets its own editor instance with its own content.
   * Also supports viewing images and other media files.
   */
  import { onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getNoteContent, saveNote, resolveEmbed } from "../services/api";
  import { createEditorExtensions, createSaveKeymap } from "../editor";
  import {
    isImageFile,
    isAudioFile,
    isVideoFile,
    isPdfFile,
    isMarkdownFile,
    isMediaFile,
    getAudioMimeType,
  } from "../utils/fileTypes";

  interface Props {
    path: string;
    readonly?: boolean;
  }

  let { path, readonly = false }: Props = $props();

  let editorContainer: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let currentPath: string | null = null;
  let isDirty = $state(false);
  let content = $state("");

  // Media viewing state
  let mediaUrl = $state<string | null>(null);
  let mediaError = $state<string | null>(null);
  let fileType = $derived(isMarkdownFile(path) ? "markdown" : isImageFile(path) ? "image" : isAudioFile(path) ? "audio" : isVideoFile(path) ? "video" : isPdfFile(path) ? "pdf" : "unknown");

  async function handleSave() {
    if (!isDirty || !currentPath) return;
    try {
      await saveNote(currentPath, content);
      isDirty = false;
    } catch (e) {
      console.error("[ColumnEditor] Failed to save:", e);
    }
  }

  async function loadMedia() {
    mediaUrl = null;
    mediaError = null;
    currentPath = path;

    try {
      // Use resolveEmbed to get the full path
      const result = await resolveEmbed({ target: path, depth: 0 });

      if (result.error) {
        mediaError = result.error;
        return;
      }

      if (result.assetUrl) {
        // Use asset:// protocol for all media
        mediaUrl = convertFileSrc(result.assetUrl);
      } else {
        mediaError = "Could not resolve media path";
      }
    } catch (e) {
      console.error("[ColumnEditor] Failed to load media:", e);
      mediaError = e instanceof Error ? e.message : "Failed to load media";
    }
  }

  async function loadAndCreateEditor() {
    if (!editorContainer || !path) return;

    // Destroy existing editor
    if (view) {
      view.destroy();
      view = null;
    }

    try {
      const noteContent = await getNoteContent(path);
      content = noteContent.content;
      currentPath = path;
      isDirty = false;

      // Update listener to track changes (local to this component)
      const updateListener = EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          content = update.state.doc.toString();
          isDirty = true;
        }
      });

      const extensions = createEditorExtensions({
        updateListener,
        saveKeymap: createSaveKeymap(handleSave),
        readonly,
      });

      const state = EditorState.create({
        doc: content,
        extensions,
      });

      view = new EditorView({
        state,
        parent: editorContainer,
      });
    } catch (e) {
      console.error("[ColumnEditor] Failed to load note:", e);
    }
  }

  // Load content when path changes
  $effect(() => {
    if (path && path !== currentPath) {
      if (isMediaFile(path)) {
        // Destroy editor if switching from markdown to media
        if (view) {
          view.destroy();
          view = null;
        }
        loadMedia();
      } else if (editorContainer) {
        // Reset media state if switching from media to markdown
        mediaUrl = null;
        mediaError = null;
        loadAndCreateEditor();
      }
    }
  });

  onDestroy(() => {
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<div class="column-editor">
  {#if fileType === "markdown"}
    <!-- Markdown editor -->
    <div class="editor-container" bind:this={editorContainer}>
      {#if isDirty}
        <div class="dirty-indicator" title="Unsaved changes">●</div>
      {/if}
    </div>
  {:else if mediaError}
    <!-- Error state -->
    <div class="media-error">
      <p>{mediaError}</p>
    </div>
  {:else if fileType === "image"}
    <!-- Image viewer -->
    <div class="media-viewer image-viewer">
      {#if mediaUrl}
        <img src={mediaUrl} alt={path} />
      {:else}
        <div class="loading">Loading image...</div>
      {/if}
    </div>
  {:else if fileType === "audio"}
    <!-- Audio player -->
    <div class="media-viewer audio-viewer">
      {#if mediaUrl}
        {@const mimeType = getAudioMimeType(path)}
        <div class="audio-container">
          <p class="media-filename">{path.split("/").pop()}</p>
          <audio controls preload="metadata">
            <source src={mediaUrl} type={mimeType} />
            Your browser does not support the audio element.
          </audio>
        </div>
      {:else}
        <div class="loading">Loading audio...</div>
      {/if}
    </div>
  {:else if fileType === "video"}
    <!-- Video player -->
    <div class="media-viewer video-viewer">
      {#if mediaUrl}
        <video controls preload="auto" src={mediaUrl}>
          Your browser does not support the video element.
        </video>
      {:else}
        <div class="loading">Loading video...</div>
      {/if}
    </div>
  {:else if fileType === "pdf"}
    <!-- PDF viewer -->
    <div class="media-viewer pdf-viewer">
      {#if mediaUrl}
        <iframe src={mediaUrl} title={path}></iframe>
      {:else}
        <div class="loading">Loading PDF...</div>
      {/if}
    </div>
  {:else}
    <!-- Unknown file type -->
    <div class="media-error">
      <p>Unsupported file type: {path}</p>
    </div>
  {/if}
</div>

<style>
  .column-editor {
    height: 100%;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .editor-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .editor-container :global(.cm-editor) {
    height: 100%;
  }

  .dirty-indicator {
    position: absolute;
    top: var(--spacing-2);
    right: var(--spacing-2);
    color: var(--color-primary);
    font-size: var(--font-size-sm);
    z-index: 10;
  }

  /* Media viewers */
  .media-viewer {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    padding: var(--spacing-4);
    background: var(--bg-surface);
  }

  .media-error {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-error);
    padding: var(--spacing-4);
  }

  .loading {
    color: var(--text-muted);
  }

  /* Image viewer */
  .image-viewer img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
  }

  /* Audio viewer */
  .audio-viewer {
    flex-direction: column;
    gap: var(--spacing-4);
  }

  .audio-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-3);
  }

  .media-filename {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    word-break: break-word;
    text-align: center;
  }

  .audio-viewer audio {
    width: 100%;
    max-width: 400px;
  }

  /* Video viewer */
  .video-viewer video {
    max-width: 100%;
    max-height: 100%;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
  }

  /* PDF viewer */
  .pdf-viewer {
    padding: 0;
  }

  .pdf-viewer iframe {
    width: 100%;
    height: 100%;
    border: none;
  }
</style>
