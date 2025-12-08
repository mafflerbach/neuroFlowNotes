<script lang="ts">
  /**
   * ColumnEditor - A self-contained editor for a single document in doc-finder mode.
   * Each column gets its own editor instance with its own content.
   * Also supports viewing images and other media files.
   */
  import { onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { defaultHighlightStyle, syntaxHighlighting } from "@codemirror/language";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getNoteContent, saveNote, resolveEmbed } from "../services/api";
  import {
    wikiLinkCompletion,
    livePreview,
    markdownHighlight,
    embedExtension,
    linkHandlerExtension,
    hoverPreviewExtension,
    pasteHandlerExtension,
  } from "../editor";

  interface Props {
    path: string;
    readonly?: boolean;
  }

  let { path, readonly = false }: Props = $props();

  // Media file extensions
  const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"];
  const AUDIO_EXTENSIONS = ["mp3", "wav", "ogg", "m4a", "flac"];
  const VIDEO_EXTENSIONS = ["mp4", "webm", "mov", "avi"];
  const PDF_EXTENSIONS = ["pdf"];

  function getFileExtension(filePath: string): string {
    return filePath.split(".").pop()?.toLowerCase() ?? "";
  }

  function isImageFile(filePath: string): boolean {
    return IMAGE_EXTENSIONS.includes(getFileExtension(filePath));
  }

  function isAudioFile(filePath: string): boolean {
    return AUDIO_EXTENSIONS.includes(getFileExtension(filePath));
  }

  function isVideoFile(filePath: string): boolean {
    return VIDEO_EXTENSIONS.includes(getFileExtension(filePath));
  }

  function isPdfFile(filePath: string): boolean {
    return PDF_EXTENSIONS.includes(getFileExtension(filePath));
  }

  function isMarkdownFile(filePath: string): boolean {
    return getFileExtension(filePath) === "md";
  }

  function isMediaFile(filePath: string): boolean {
    return isImageFile(filePath) || isAudioFile(filePath) || isVideoFile(filePath) || isPdfFile(filePath);
  }

  let editorContainer: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let currentPath: string | null = null;
  let isDirty = $state(false);
  let content = $state("");

  // Media viewing state
  let mediaUrl = $state<string | null>(null);
  let mediaError = $state<string | null>(null);
  let fileType = $derived(isMarkdownFile(path) ? "markdown" : isImageFile(path) ? "image" : isAudioFile(path) ? "audio" : isVideoFile(path) ? "video" : isPdfFile(path) ? "pdf" : "unknown");

  // Create a custom theme
  const editorTheme = EditorView.theme({
    "&": {
      height: "100%",
      fontSize: "var(--font-size-md)",
      color: "var(--text-primary)",
    },
    ".cm-content": {
      fontFamily: "var(--font-family-mono)",
      padding: "var(--spacing-4) 0",
      color: "var(--text-primary)",
    },
    ".cm-line": {
      color: "var(--text-primary)",
      padding: "0 var(--spacing-4)",
    },
    ".cm-gutters": {
      background: "var(--editor-gutter-bg)",
      border: "none",
      color: "var(--editor-gutter-color)",
    },
    ".cm-activeLineGutter": {
      background: "var(--editor-active-gutter-bg)",
    },
    ".cm-activeLine": {
      background: "var(--editor-active-line-bg)",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "var(--editor-cursor)",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      background: "var(--editor-selection-bg)",
    },
  });

  // Update listener to track changes
  function createUpdateListener() {
    return EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        content = update.state.doc.toString();
        isDirty = true;
      }
    });
  }

  // Save on Cmd/Ctrl+S
  function createSaveKeymap() {
    return keymap.of([
      {
        key: "Mod-s",
        run: () => {
          handleSave();
          return true;
        },
      },
    ]);
  }

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
        // preload="auto" on audio/video elements will load the full file for duration detection
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

      const extensions = [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        history(),
        highlightSelectionMatches(),
        markdown({
          base: markdownLanguage,
          codeLanguages: languages,
        }),
        markdownHighlight(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
        createSaveKeymap(),
        createUpdateListener(),
        editorTheme,
        EditorView.lineWrapping,
        wikiLinkCompletion(),
        livePreview(),
        embedExtension(),
        linkHandlerExtension(),
        hoverPreviewExtension(),
        pasteHandlerExtension(),
      ];

      if (readonly) {
        extensions.push(EditorState.readOnly.of(true));
      }

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
        {@const ext = path.split(".").pop()?.toLowerCase() || ""}
        {@const mimeType = { mp3: "audio/mpeg", wav: "audio/wav", ogg: "audio/ogg", m4a: "audio/mp4", flac: "audio/flac", aac: "audio/aac" }[ext] || "audio/mpeg"}
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
