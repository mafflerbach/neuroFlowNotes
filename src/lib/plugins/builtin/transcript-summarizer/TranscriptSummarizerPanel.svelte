<script lang="ts">
  /**
   * Transcript Summarizer Panel
   *
   * Sidebar panel for processing YouTube transcript files using a local LLM.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { pluginRegistry } from "../../registry.svelte";
  import type { TranscriptSummarizerSettings, SummarizerResult, SummarizerProgress } from "./types";
  import { Video, Sparkles, Loader2, CheckCircle, XCircle, AlertCircle, RefreshCw } from "lucide-svelte";

  // Get plugin settings
  const plugin = $derived(pluginRegistry.get<TranscriptSummarizerSettings>("transcript-summarizer"));
  const settings = $derived(plugin?.config.settings);

  // State
  let pendingCount = $state<number | null>(null);
  let isProcessing = $state(false);
  let isChecking = $state(false);
  let error = $state<string | null>(null);
  let result = $state<SummarizerResult | null>(null);
  let progressMessages = $state<SummarizerProgress[]>([]);

  const hasValidSettings = $derived(
    settings && settings.inputDir && settings.outputDir && settings.lmStudioEndpoint
  );

  // Check pending count when settings become available
  $effect(() => {
    if (settings?.inputDir) {
      checkPendingCount();
    }
  });

  async function checkPendingCount() {
    if (!settings?.inputDir) return;

    isChecking = true;
    try {
      pendingCount = await invoke<number>("count_pending_transcripts", {
        inputDir: settings.inputDir,
      });
    } catch (e) {
      console.error("Failed to count pending transcripts:", e);
      pendingCount = null;
    } finally {
      isChecking = false;
    }
  }

  async function runSummarizer() {
    if (!settings) return;

    isProcessing = true;
    error = null;
    result = null;
    progressMessages = [];

    try {
      const summarizerResult = await invoke<SummarizerResult>("run_transcript_summarizer", {
        inputDir: settings.inputDir,
        outputDir: settings.outputDir,
        endpoint: settings.lmStudioEndpoint,
        model: settings.model || null,
        assetTemplate: settings.assetTemplate || null,
      });

      result = summarizerResult;

      // Parse progress messages from output
      for (const line of summarizerResult.output_lines) {
        try {
          const msg = JSON.parse(line) as SummarizerProgress;
          progressMessages = [...progressMessages, msg];
        } catch {
          // Not JSON, ignore
        }
      }

      if (!summarizerResult.success && summarizerResult.failed > 0) {
        error = `${summarizerResult.failed} transcript(s) failed to process`;
      }

      // Refresh pending count
      await checkPendingCount();
    } catch (e) {
      console.error("Transcript summarizer error:", e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  function clearResults() {
    result = null;
    progressMessages = [];
    error = null;
  }
</script>

<div class="summarizer-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <Video size={16} />
      Transcript Summarizer
    </h3>
  </div>

  <div class="panel-content">
    <!-- Settings Check -->
    {#if !hasValidSettings}
      <div class="warning-message">
        <AlertCircle size={16} />
        <span>Please configure Input Directory, Output Directory, and LM Studio Endpoint in plugin settings.</span>
      </div>
    {/if}

    <!-- Pending Count -->
    <div class="status-card">
      <div class="status-header">
        <span class="status-label">Pending Transcripts</span>
        <button
          class="refresh-btn"
          onclick={checkPendingCount}
          disabled={isChecking || isProcessing || !hasValidSettings}
          title="Refresh count"
        >
          <RefreshCw size={14} class={isChecking ? "spinning" : ""} />
        </button>
      </div>
      <div class="status-value">
        {#if pendingCount === null}
          <span class="muted">--</span>
        {:else if pendingCount === 0}
          <span class="success">No pending transcripts</span>
        {:else}
          <span class="pending">{pendingCount} file{pendingCount !== 1 ? "s" : ""}</span>
        {/if}
      </div>
      {#if settings?.inputDir}
        <div class="status-path">{settings.inputDir}</div>
      {/if}
    </div>

    <!-- Run Button -->
    <button
      class="generate-btn"
      onclick={runSummarizer}
      disabled={isProcessing || !hasValidSettings || pendingCount === 0}
    >
      {#if isProcessing}
        <Loader2 size={16} class="spinning" />
        Processing Transcripts...
      {:else}
        <Sparkles size={16} />
        Process {pendingCount ?? 0} Transcript{pendingCount !== 1 ? "s" : ""}
      {/if}
    </button>

    <!-- Error Display -->
    {#if error}
      <div class="error-message">
        <XCircle size={16} />
        {error}
      </div>
    {/if}

    <!-- Results -->
    {#if result}
      <div class="results-section">
        <div class="results-header">
          <span class="results-title">Results</span>
          <button class="clear-btn" onclick={clearResults}>Clear</button>
        </div>

        <div class="results-summary">
          <div class="stat success">
            <CheckCircle size={14} />
            <span>{result.processed} processed</span>
          </div>
          {#if result.failed > 0}
            <div class="stat failed">
              <XCircle size={14} />
              <span>{result.failed} failed</span>
            </div>
          {/if}
        </div>

        <!-- Progress Details -->
        {#if progressMessages.length > 0}
          <div class="progress-list">
            {#each progressMessages.filter((m) => m.type === "success" || m.type === "error") as msg}
              <div class="progress-item {msg.type}">
                {#if msg.type === "success"}
                  <CheckCircle size={12} />
                  <div class="progress-details">
                    <span class="progress-file">{msg.output?.split("/").pop() || msg.source?.split("/").pop()}</span>
                    {#if msg.video_id}
                      <span class="progress-meta">ID: {msg.video_id}</span>
                    {/if}
                  </div>
                {:else}
                  <XCircle size={12} />
                  <span class="progress-error">{msg.error || msg.file}</span>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Requirements Info -->
    <div class="info-section">
      <details>
        <summary>Requirements</summary>
        <ul>
          <li>Python 3 with requests installed</li>
          <li>LM Studio running locally</li>
          <li>Transcript files in input directory</li>
        </ul>
      </details>
    </div>
  </div>
</div>

<style>
  .summarizer-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
  }

  .panel-header {
    padding: var(--spacing-3);
    border-bottom: 1px solid var(--border-light);
  }

  .panel-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-3);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .status-card {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
    padding: var(--spacing-3);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-light);
  }

  .status-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .status-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .status-value {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
  }

  .status-value .muted {
    color: var(--text-muted);
  }

  .status-value .success {
    color: var(--color-success, #059669);
  }

  .status-value .pending {
    color: var(--color-primary);
  }

  .status-path {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .generate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .generate-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .generate-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .warning-message {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--color-warning-bg, #fef3c7);
    color: var(--color-warning-text, #92400e);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
  }

  .error-message {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--color-error-bg, #fee2e2);
    color: var(--color-error-text, #991b1b);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
  }

  .results-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
  }

  .results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .results-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .clear-btn {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-xs);
    background: var(--bg-primary);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
  }

  .results-summary {
    display: flex;
    gap: var(--spacing-3);
  }

  .stat {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
  }

  .stat.success {
    color: var(--color-success, #059669);
  }

  .stat.failed {
    color: var(--color-error, #dc2626);
  }

  .progress-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
    max-height: 200px;
    overflow-y: auto;
  }

  .progress-item {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-1);
    font-size: var(--font-size-xs);
    padding: var(--spacing-1);
    border-radius: var(--radius-sm);
  }

  .progress-item.success {
    color: var(--color-success, #059669);
    background: var(--color-success-bg, #d1fae5);
  }

  .progress-item.error {
    color: var(--color-error, #dc2626);
    background: var(--color-error-bg, #fee2e2);
  }

  .progress-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .progress-file {
    word-break: break-all;
  }

  .progress-meta {
    font-size: 10px;
    opacity: 0.8;
  }

  .progress-error {
    word-break: break-all;
  }

  .info-section {
    margin-top: auto;
    padding-top: var(--spacing-2);
    border-top: 1px solid var(--border-light);
  }

  .info-section details {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .info-section summary {
    cursor: pointer;
    font-weight: var(--font-weight-medium);
  }

  .info-section ul {
    margin: var(--spacing-1) 0 0 var(--spacing-3);
    padding: 0;
  }

  .info-section li {
    margin: var(--spacing-1) 0;
  }
</style>
