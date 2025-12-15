<script lang="ts">
  /**
   * LLM Summarizer Panel
   *
   * Sidebar panel for generating AI summaries of schedule blocks and notes.
   */
  import { pluginRegistry } from "../../registry.svelte";
  import type { LLMSummarizerSettings, BlockWithContent } from "./types";
  import type { ScheduleBlock, HttpResponse } from "../../types";
  import { Calendar, Sparkles, Copy, Check, Loader2, RefreshCw } from "lucide-svelte";

  // Get plugin settings and backend hooks (lazy access)
  const plugin = $derived(pluginRegistry.get<LLMSummarizerSettings>("llm-daily-summarizer"));
  const settings = $derived(plugin?.config.settings);
  const hooks = $derived(pluginRegistry.getBackendHooks());

  // State
  let selectedDate = $state(new Date().toISOString().split("T")[0]);
  let outputFormat = $state<"daily" | "confluence" | "standup">("daily");
  let isLoading = $state(false);
  let isCopied = $state(false);
  let error = $state<string | null>(null);
  let blocks = $state<BlockWithContent[]>([]);
  let summary = $state<string | null>(null);
  let tokensUsed = $state<number | null>(null);

  // Format labels
  const formatLabels = {
    daily: "Daily Summary",
    confluence: "Confluence Format",
    standup: "Standup Update",
  };

  // Load schedule blocks when date changes or hooks become available
  $effect(() => {
    if (selectedDate && hooks) {
      loadBlocks();
    }
  });

  async function loadBlocks() {
    if (!selectedDate || !hooks) {
      error = hooks ? null : "Plugin system not initialized";
      return;
    }

    error = null;

    try {
      const scheduleBlocks = await hooks.getScheduleBlocks(selectedDate, selectedDate);

      // Fetch note content for each block
      const blocksWithContent: BlockWithContent[] = await Promise.all(
        scheduleBlocks.map(async (block: ScheduleBlock) => {
          let noteContent: { path: string; title: string | null; content: string } | null = null;

          if (block.note_id) {
            noteContent = await hooks.getNoteContent(block.note_id);
          }

          return {
            blockId: block.id,
            date: block.date,
            startTime: block.start_time,
            endTime: block.end_time,
            label: block.label,
            noteId: block.note_id,
            notePath: noteContent?.path ?? null,
            noteTitle: noteContent?.title ?? null,
            noteContent: noteContent?.content ?? null,
          };
        })
      );

      blocks = blocksWithContent.sort((a, b) => a.startTime.localeCompare(b.startTime));
    } catch (e) {
      console.error("Failed to load blocks:", e);
      error = "Failed to load schedule blocks";
    }
  }

  async function generateSummary() {
    if (!settings || blocks.length === 0) return;

    isLoading = true;
    error = null;
    summary = null;

    try {
      // Build the prompt
      const systemPrompt = getSystemPrompt();
      const userPrompt = buildUserPrompt();

      // Call LLM API
      const response = await callLLM(systemPrompt, userPrompt);

      summary = response.content;
      tokensUsed = response.tokens;
    } catch (e) {
      console.error("Failed to generate summary:", e);
      error = e instanceof Error ? e.message : "Failed to generate summary";
    } finally {
      isLoading = false;
    }
  }

  function getSystemPrompt(): string {
    if (!settings) return "";

    switch (outputFormat) {
      case "daily":
        return settings.dailySummaryPrompt;
      case "confluence":
        return settings.confluencePrompt;
      case "standup":
        return settings.standupPrompt;
    }
  }

  function buildUserPrompt(): string {
    const lines: string[] = [];
    lines.push(`Date: ${selectedDate}`);
    lines.push("");
    lines.push("Schedule blocks for the day:");
    lines.push("");

    for (const block of blocks) {
      lines.push(`## ${block.startTime} - ${block.endTime}: ${block.label || "Untitled"}`);

      if (block.noteTitle) {
        lines.push(`Linked note: ${block.noteTitle}`);
      }

      if (block.noteContent) {
        lines.push("");
        lines.push("Note content:");
        lines.push("```");
        // Truncate very long content
        const content = block.noteContent.length > 3000
          ? block.noteContent.substring(0, 3000) + "\n... (truncated)"
          : block.noteContent;
        lines.push(content);
        lines.push("```");
      }

      lines.push("");
    }

    return lines.join("\n");
  }

  async function callLLM(systemPrompt: string, userPrompt: string): Promise<{ content: string; tokens: number | null }> {
    if (!settings) throw new Error("Settings not available");
    if (!hooks) throw new Error("Plugin system not initialized");

    const endpoint = settings.endpoint.endsWith("/")
      ? settings.endpoint.slice(0, -1)
      : settings.endpoint;

    const requestBody = {
      model: settings.model || undefined,
      messages: [
        { role: "system", content: systemPrompt },
        { role: "user", content: userPrompt },
      ],
      max_tokens: settings.maxTokens,
      temperature: settings.temperature,
    };

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (settings.apiKey) {
      headers["Authorization"] = `Bearer ${settings.apiKey}`;
    }

    const response: HttpResponse = await hooks.httpRequest({
      url: `${endpoint}/chat/completions`,
      method: "POST",
      headers,
      body: requestBody,
      timeout: 120000, // 2 minute timeout for LLM responses
    });

    if (response.status !== 200) {
      const errorBody = response.body as { error?: { message?: string } };
      throw new Error(errorBody?.error?.message || `HTTP ${response.status}`);
    }

    const data = response.body as {
      choices?: { message?: { content?: string } }[];
      usage?: { total_tokens?: number };
    };

    const content = data.choices?.[0]?.message?.content;
    if (!content) {
      throw new Error("No response content from LLM");
    }

    return {
      content,
      tokens: data.usage?.total_tokens ?? null,
    };
  }

  async function copyToClipboard() {
    if (!summary) return;

    try {
      await navigator.clipboard.writeText(summary);
      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function formatTime(time: string): string {
    // Convert HH:MM:SS to HH:MM
    return time.substring(0, 5);
  }
</script>

<div class="summarizer-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <Sparkles size={16} />
      AI Daily Summary
    </h3>
  </div>

  <div class="panel-content">
    <!-- Date Selector -->
    <div class="control-group">
      <label class="control-label" for="summary-date">
        <Calendar size={14} />
        Date
      </label>
      <input
        type="date"
        id="summary-date"
        class="date-input"
        bind:value={selectedDate}
      />
    </div>

    <!-- Format Selector -->
    <div class="control-group">
      <label class="control-label">Output Format</label>
      <div class="format-buttons">
        {#each Object.entries(formatLabels) as [value, label]}
          <button
            class="format-btn"
            class:active={outputFormat === value}
            onclick={() => outputFormat = value as "daily" | "confluence" | "standup"}
          >
            {label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Blocks Preview -->
    <div class="control-group">
      <div class="blocks-header">
        <span class="control-label">Schedule Blocks ({blocks.length})</span>
        <button class="refresh-btn" onclick={loadBlocks} title="Refresh blocks">
          <RefreshCw size={14} />
        </button>
      </div>

      {#if blocks.length === 0}
        <p class="no-blocks">No schedule blocks for this date.</p>
      {:else}
        <div class="blocks-list">
          {#each blocks as block}
            <div class="block-item">
              <span class="block-time">{formatTime(block.startTime)}-{formatTime(block.endTime)}</span>
              <span class="block-label">{block.label || "Untitled"}</span>
              {#if block.noteId}
                <span class="has-note" title="Has linked note">+</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Generate Button -->
    <button
      class="generate-btn"
      onclick={generateSummary}
      disabled={isLoading || blocks.length === 0 || !settings}
    >
      {#if isLoading}
        <Loader2 size={16} class="spinning" />
        Generating...
      {:else}
        <Sparkles size={16} />
        Generate Summary
      {/if}
    </button>

    <!-- Error Display -->
    {#if error}
      <div class="error-message">
        {error}
      </div>
    {/if}

    <!-- Summary Output -->
    {#if summary}
      <div class="summary-section">
        <div class="summary-header">
          <span class="summary-title">Generated Summary</span>
          <div class="summary-actions">
            {#if tokensUsed}
              <span class="token-count">{tokensUsed} tokens</span>
            {/if}
            <button class="copy-btn" onclick={copyToClipboard} title="Copy to clipboard">
              {#if isCopied}
                <Check size={14} />
              {:else}
                <Copy size={14} />
              {/if}
            </button>
          </div>
        </div>
        <div class="summary-content">
          {summary}
        </div>
      </div>
    {/if}
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
    gap: var(--spacing-2);
  }

  .control-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .date-input {
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .date-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .format-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-1);
  }

  .format-btn {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    background: var(--bg-surface);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .format-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .format-btn.active {
    background: var(--color-primary);
    color: var(--btn-primary-text);
    border-color: var(--color-primary);
  }

  .blocks-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .refresh-btn {
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .refresh-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .no-blocks {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-style: italic;
    margin: 0;
  }

  .blocks-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
    max-height: 150px;
    overflow-y: auto;
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
  }

  .block-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    font-size: var(--font-size-xs);
  }

  .block-time {
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
  }

  .block-label {
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .has-note {
    color: var(--color-primary);
    font-weight: var(--font-weight-semibold);
  }

  .generate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--btn-primary-text);
    background: var(--color-primary);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .generate-btn:hover:not(:disabled) {
    background: var(--btn-primary-bg-hover);
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

  .error-message {
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    color: var(--color-error);
    background: var(--color-error-bg);
    border-radius: var(--radius-sm);
  }

  .summary-section {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
    border-bottom: 1px solid var(--border-light);
  }

  .summary-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .summary-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .token-count {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .copy-btn {
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .summary-content {
    padding: var(--spacing-3);
    font-size: var(--font-size-sm);
    line-height: var(--line-height-relaxed);
    color: var(--text-primary);
    white-space: pre-wrap;
    max-height: 300px;
    overflow-y: auto;
  }
</style>
