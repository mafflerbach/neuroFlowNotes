<script lang="ts">
  import { X, Copy, Check } from "lucide-svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { isImageFile, isAudioFile, isVideoFile } from "../utils/fileTypes";

  interface Props {
    path: string;
    vaultPath: string;
    onClose: () => void;
  }

  let { path, vaultPath, onClose }: Props = $props();

  let copied = $state(false);

  // Get the filename from the path
  const filename = $derived(path.split("/").pop() || path);

  // Build the full file path and convert to asset URL
  const assetUrl = $derived.by(() => {
    const fullPath = `${vaultPath}/${path}`;
    return convertFileSrc(fullPath);
  });

  // Determine media type
  const isImage = $derived(isImageFile(filename));
  const isAudio = $derived(isAudioFile(filename));
  const isVideo = $derived(isVideoFile(filename));

  async function copyWikiLink() {
    const wikiLink = `![[${filename}]]`;
    try {
      await navigator.clipboard.writeText(wikiLink);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="media-viewer">
  <div class="header">
    <h2 class="filename">{filename}</h2>
    <div class="actions">
      <button class="action-btn" onclick={copyWikiLink} title="Copy wiki link">
        {#if copied}
          <Check size={16} />
        {:else}
          <Copy size={16} />
        {/if}
      </button>
      <button class="action-btn close-btn" onclick={onClose} title="Close">
        <X size={16} />
      </button>
    </div>
  </div>

  <div class="content">
    {#if isImage}
      <div class="image-container">
        <img src={assetUrl} alt={filename} />
      </div>
    {:else if isAudio}
      <div class="audio-container">
        <audio controls src={assetUrl}>
          Your browser does not support the audio element.
        </audio>
      </div>
    {:else if isVideo}
      <div class="video-container">
        <video controls src={assetUrl}>
          Your browser does not support the video element.
        </video>
      </div>
    {:else}
      <div class="unsupported">
        <p>Preview not available for this file type.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .media-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface-raised);
  }

  .filename {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-4);
    overflow: auto;
  }

  .image-container {
    max-width: 100%;
    max-height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-container img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
  }

  .audio-container {
    width: 100%;
    max-width: 500px;
    padding: var(--spacing-6);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
  }

  .audio-container audio {
    width: 100%;
  }

  .video-container {
    max-width: 100%;
    max-height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .video-container video {
    max-width: 100%;
    max-height: 100%;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
  }

  .unsupported {
    text-align: center;
    color: var(--text-muted);
  }
</style>
