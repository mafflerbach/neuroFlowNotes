<script lang="ts">
  import { Modal } from "./shared";
  import { FolderOpen, Check, AlertTriangle, Loader2 } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { importObsidianVault } from "../services/api";
  import { onImportProgress } from "../services/events";
  import { vaultStore } from "../stores";
  import type { ImportProgress, ImportResult } from "../types";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open: isOpen, onClose }: Props = $props();

  // State
  let sourcePath = $state("");
  let targetSubfolder = $state("");
  let isImporting = $state(false);
  let progress = $state<ImportProgress | null>(null);
  let result = $state<ImportResult | null>(null);
  let error = $state<string | null>(null);

  // Progress listener
  let unlistenProgress: UnlistenFn | null = null;

  // Reset state when modal opens
  $effect(() => {
    if (isOpen) {
      sourcePath = "";
      targetSubfolder = "";
      isImporting = false;
      progress = null;
      result = null;
      error = null;
    }
  });

  // Cleanup listener on close
  $effect(() => {
    return () => {
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    };
  });

  async function selectSourceFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Markdown Folder",
      });

      if (selected && typeof selected === "string") {
        sourcePath = selected;
      }
    } catch (e) {
      console.error("Failed to select folder:", e);
    }
  }

  async function startImport() {
    if (!sourcePath) return;

    isImporting = true;
    progress = null;
    result = null;
    error = null;

    // Listen for progress events
    unlistenProgress = await onImportProgress((p) => {
      progress = p;
    });

    try {
      const importResult = await importObsidianVault({
        source_path: sourcePath,
        target_subfolder: targetSubfolder || null,
      });

      result = importResult;

      // Refresh the vault after import
      await vaultStore.refreshFolderTree();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isImporting = false;
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    }
  }

  function handleClose() {
    if (!isImporting) {
      onClose();
    }
  }

  const progressPercent = $derived(
    progress && progress.total_files > 0
      ? Math.round((progress.files_processed / progress.total_files) * 100)
      : 0
  );
</script>

<Modal open={isOpen} title="Import Markdown Folder" onClose={handleClose} maxWidth="520px">
  <div class="import-content">
    {#if result}
      <!-- Success state -->
      <div class="result-state">
        <div class="result-icon success">
          <Check size={32} />
        </div>
        <h3 class="result-title">Import Complete</h3>
        <div class="result-stats">
          <div class="stat">
            <span class="stat-value">{result.notes_imported}</span>
            <span class="stat-label">notes imported</span>
          </div>
          <div class="stat">
            <span class="stat-value">{result.files_copied}</span>
            <span class="stat-label">files copied</span>
          </div>
          <div class="stat">
            <span class="stat-value">{result.properties_imported}</span>
            <span class="stat-label">properties</span>
          </div>
          <div class="stat">
            <span class="stat-value">{result.tags_imported}</span>
            <span class="stat-label">tags</span>
          </div>
        </div>
        <p class="result-time">Completed in {(result.duration_ms / 1000).toFixed(1)}s</p>
        {#if result.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings ({result.warnings.length})</h4>
            <ul class="warning-list">
              {#each result.warnings.slice(0, 5) as warning}
                <li>{warning}</li>
              {/each}
              {#if result.warnings.length > 5}
                <li class="more">...and {result.warnings.length - 5} more</li>
              {/if}
            </ul>
          </div>
        {/if}
      </div>
    {:else if error}
      <!-- Error state -->
      <div class="result-state">
        <div class="result-icon error">
          <AlertTriangle size={32} />
        </div>
        <h3 class="result-title">Import Failed</h3>
        <p class="error-message">{error}</p>
      </div>
    {:else if isImporting}
      <!-- Progress state -->
      <div class="progress-state">
        <div class="progress-header">
          <Loader2 size={20} class="spinner" />
          <span>Importing...</span>
        </div>
        {#if progress}
          <div class="progress-bar-container">
            <div class="progress-bar" style="width: {progressPercent}%"></div>
          </div>
          <div class="progress-details">
            <span class="progress-percent">{progressPercent}%</span>
            <span class="progress-count">{progress.files_processed} / {progress.total_files} files</span>
          </div>
          <p class="current-file" title={progress.current_file}>
            {progress.current_file.split("/").pop()}
          </p>
          <div class="progress-stats">
            <span>{progress.properties_imported} properties</span>
            <span>{progress.tags_imported} tags</span>
          </div>
        {/if}
      </div>
    {:else}
      <!-- Initial state -->
      <div class="form-section">
        <label class="form-label">Source Folder</label>
        <div class="folder-select">
          <input
            type="text"
            class="folder-input"
            placeholder="Select folder with markdown files..."
            value={sourcePath}
            readonly
          />
          <button class="folder-btn" onclick={selectSourceFolder}>
            <FolderOpen size={16} />
            Browse
          </button>
        </div>
        <p class="form-hint">Select any folder containing markdown files (works with Obsidian vaults too)</p>
      </div>

      <div class="form-section">
        <label class="form-label" for="target-subfolder">Target Subfolder (optional)</label>
        <input
          id="target-subfolder"
          type="text"
          class="text-input"
          placeholder="e.g., imported/obsidian"
          bind:value={targetSubfolder}
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
        />
        <p class="form-hint">Leave empty to import at the vault root</p>
      </div>

      <div class="info-box">
        <h4>What will be imported:</h4>
        <ul>
          <li>All markdown files with folder structure preserved</li>
          <li>YAML frontmatter converted to properties</li>
          <li>Tags from frontmatter merged with inline tags</li>
          <li>Images and other assets copied</li>
        </ul>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    {#if result || error}
      <button class="btn btn-primary" onclick={handleClose}>Done</button>
    {:else if isImporting}
      <button class="btn btn-secondary" disabled>Importing...</button>
    {:else}
      <button class="btn btn-secondary" onclick={handleClose}>Cancel</button>
      <button class="btn btn-primary" onclick={startImport} disabled={!sourcePath}>
        Start Import
      </button>
    {/if}
  {/snippet}
</Modal>

<style>
  .import-content {
    min-height: 200px;
  }

  .form-section {
    margin-bottom: var(--spacing-4);
  }

  .form-label {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin-bottom: var(--spacing-2);
  }

  .folder-select {
    display: flex;
    gap: var(--spacing-2);
  }

  .folder-input {
    flex: 1;
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-md);
    background: var(--input-bg);
    color: var(--input-text);
    cursor: default;
  }

  .folder-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--btn-secondary-text);
    background: var(--btn-secondary-bg);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    white-space: nowrap;
  }

  .folder-btn:hover {
    background: var(--btn-secondary-bg-hover);
  }

  .text-input {
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-md);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .text-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .form-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: var(--spacing-1) 0 0 0;
  }

  .info-box {
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
  }

  .info-box h4 {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin: 0 0 var(--spacing-2) 0;
  }

  .info-box ul {
    margin: 0;
    padding-left: var(--spacing-4);
    color: var(--text-secondary);
  }

  .info-box li {
    margin-bottom: var(--spacing-1);
  }

  /* Progress state */
  .progress-state {
    text-align: center;
    padding: var(--spacing-4) 0;
  }

  .progress-header {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin-bottom: var(--spacing-4);
  }

  .progress-header :global(.spinner) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .progress-bar-container {
    height: 8px;
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-full);
    overflow: hidden;
    margin-bottom: var(--spacing-2);
  }

  .progress-bar {
    height: 100%;
    background: var(--color-primary);
    border-radius: var(--radius-full);
    transition: width 0.2s ease;
  }

  .progress-details {
    display: flex;
    justify-content: space-between;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: var(--spacing-2);
  }

  .progress-percent {
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .current-file {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0 0 var(--spacing-2) 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .progress-stats {
    display: flex;
    justify-content: center;
    gap: var(--spacing-4);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  /* Result state */
  .result-state {
    text-align: center;
    padding: var(--spacing-4) 0;
  }

  .result-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    margin: 0 auto var(--spacing-3);
    border-radius: var(--radius-full);
  }

  .result-icon.success {
    background: var(--color-success-light, rgba(34, 197, 94, 0.1));
    color: var(--color-success, #22c55e);
  }

  .result-icon.error {
    background: var(--color-error-light, rgba(239, 68, 68, 0.1));
    color: var(--color-error, #ef4444);
  }

  .result-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0 0 var(--spacing-3) 0;
  }

  .result-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--spacing-3);
    margin-bottom: var(--spacing-3);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .stat-value {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .stat-label {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .result-time {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .warnings {
    margin-top: var(--spacing-4);
    padding: var(--spacing-3);
    background: var(--color-warning-light, rgba(245, 158, 11, 0.1));
    border-radius: var(--radius-md);
    text-align: left;
  }

  .warnings h4 {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-warning, #f59e0b);
    margin: 0 0 var(--spacing-2) 0;
  }

  .warning-list {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0;
    padding-left: var(--spacing-4);
  }

  .warning-list li {
    margin-bottom: var(--spacing-1);
  }

  .warning-list .more {
    color: var(--text-muted);
    font-style: italic;
  }

  .error-message {
    font-size: var(--font-size-base);
    color: var(--color-error, #ef4444);
    margin: 0;
  }

  /* Footer buttons */
  .btn {
    padding: var(--spacing-2) var(--spacing-4);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-normal);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    color: var(--btn-secondary-text);
    background: var(--btn-secondary-bg);
    border: none;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--btn-secondary-bg-hover);
  }

  .btn-primary {
    color: var(--btn-primary-text);
    background: var(--btn-primary-bg);
    border: none;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--btn-primary-bg-hover);
  }
</style>
