<script lang="ts">
  /**
   * Link Summarizer Panel
   *
   * Sidebar panel for summarizing web pages using a local LLM.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { pluginRegistry } from "../../registry.svelte";
  import type { LinkSummarizerSettings, SummarizerResult, SummarizerProgress } from "./types";
  import { Link, Sparkles, Loader2, CheckCircle, XCircle, AlertCircle } from "lucide-svelte";

  // Get plugin settings
  const plugin = $derived(pluginRegistry.get<LinkSummarizerSettings>("link-summarizer"));
  const settings = $derived(plugin?.config.settings);

  // State
  let urlInput = $state("");
  let isProcessing = $state(false);
  let error = $state<string | null>(null);
  let result = $state<SummarizerResult | null>(null);
  let progressMessages = $state<SummarizerProgress[]>([]);

  // Parse URLs from input
  const urls = $derived(
    urlInput
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.startsWith("http"))
  );

  const urlCount = $derived(urls.length);
  const hasValidUrls = $derived(urlCount > 0);
  const hasValidSettings = $derived(
    settings && settings.outputDir && settings.lmStudioEndpoint
  );

  async function runSummarizer() {
    if (!settings || !hasValidUrls) return;

    isProcessing = true;
    error = null;
    result = null;
    progressMessages = [];

    try {
      const summarizerResult = await invoke<SummarizerResult>("run_link_summarizer", {
        urls,
        outputDir: settings.outputDir,
        endpoint: settings.lmStudioEndpoint,
        model: settings.model || null,
        tags: settings.defaultTags || null,
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
        error = `${summarizerResult.failed} URL(s) failed to process`;
      }
    } catch (e) {
      console.error("Link summarizer error:", e);
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
      <Link size={16} />
      Link Summarizer
    </h3>
  </div>

  <div class="panel-content">
    <!-- Settings Check -->
    {#if !hasValidSettings}
      <div class="warning-message">
        <AlertCircle size={16} />
        <span>Please configure Output Directory and LM Studio Endpoint in plugin settings.</span>
      </div>
    {/if}

    <!-- URL Input -->
    <div class="control-group">
      <label class="control-label" for="url-input">
        URLs to Summarize
      </label>
      <textarea
        id="url-input"
        class="url-textarea"
        placeholder="Paste URLs here, one per line...&#10;https://example.com/article1&#10;https://example.com/article2"
        bind:value={urlInput}
        disabled={isProcessing}
        rows={8}
      ></textarea>
      <span class="url-count">
        {urlCount} valid URL{urlCount !== 1 ? "s" : ""} detected
      </span>
    </div>

    <!-- Run Button -->
    <button
      class="generate-btn"
      onclick={runSummarizer}
      disabled={isProcessing || !hasValidUrls || !hasValidSettings}
    >
      {#if isProcessing}
        <Loader2 size={16} class="spinning" />
        Processing {urlCount} URL{urlCount !== 1 ? "s" : ""}...
      {:else}
        <Sparkles size={16} />
        Summarize {urlCount} Link{urlCount !== 1 ? "s" : ""}
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
                  <span class="progress-title">{msg.title || msg.url}</span>
                {:else}
                  <XCircle size={12} />
                  <span class="progress-error">{msg.error || msg.url}</span>
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
          <li>Node.js with puppeteer and axios installed</li>
          <li>LM Studio running locally</li>
          <li>Valid output directory path</li>
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

  .control-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .control-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .url-textarea {
    width: 100%;
    padding: var(--spacing-2);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-md);
    background: var(--bg-primary);
    color: var(--text-primary);
    resize: vertical;
    min-height: 120px;
  }

  .url-textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .url-textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .url-count {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
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

  .progress-title,
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
