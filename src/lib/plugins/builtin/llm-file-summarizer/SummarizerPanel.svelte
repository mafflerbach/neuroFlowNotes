<script lang="ts">
  /**
   * LLM File Summarizer Panel
   *
   * Sidebar panel for generating AI summaries of individual files.
   */
  import { pluginRegistry } from "../../registry.svelte";
  import type { LLMSummarizerSettings } from "./types";
  import type { HttpResponse, NoteListItem, NoteContent } from "../../types";
  import { FileText, Sparkles, Copy, Check, Loader2, Search, X } from "lucide-svelte";

  // Get plugin settings and backend hooks (lazy access)
  const plugin = $derived(pluginRegistry.get<LLMSummarizerSettings>("llm-file-summarizer"));
  const settings = $derived(plugin?.config.settings);
  const hooks = $derived(pluginRegistry.getBackendHooks());

  // State
  let notes = $state<NoteListItem[]>([]);
  let selectedNote = $state<NoteListItem | null>(null);
  let fileContent = $state<NoteContent | null>(null);
  let searchQuery = $state("");
  let isLoading = $state(false);
  let isCopied = $state(false);
  let error = $state<string | null>(null);
  let summary = $state<string | null>(null);
  let tokensUsed = $state<number | null>(null);
  let showDropdown = $state(false);

  // Filtered notes based on search
  const filteredNotes = $derived(
    searchQuery.trim()
      ? notes.filter((n) => {
          const query = searchQuery.toLowerCase();
          return (
            n.path.toLowerCase().includes(query) ||
            (n.title?.toLowerCase().includes(query) ?? false)
          );
        }).slice(0, 20)
      : notes.slice(0, 20)
  );

  // Load notes list when hooks become available
  $effect(() => {
    if (hooks) {
      loadNotes();
    }
  });

  async function loadNotes() {
    if (!hooks) return;
    try {
      notes = await hooks.listNotes();
    } catch (e) {
      console.error("Failed to load notes:", e);
    }
  }

  async function selectNote(note: NoteListItem) {
    selectedNote = note;
    showDropdown = false;
    searchQuery = "";
    error = null;
    summary = null;

    if (!hooks) return;

    try {
      fileContent = await hooks.getNoteByPath(note.path);
    } catch (e) {
      console.error("Failed to load note content:", e);
      error = "Failed to load file content";
    }
  }

  function clearSelection() {
    selectedNote = null;
    fileContent = null;
    summary = null;
    error = null;
  }

  async function generateSummary() {
    if (!settings || !fileContent) return;

    isLoading = true;
    error = null;
    summary = null;

    try {
      const systemPrompt = settings.dailySummaryPrompt; // Reuse the prompt
      const userPrompt = buildUserPrompt();

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

  function buildUserPrompt(): string {
    if (!fileContent) return "";

    const lines: string[] = [];
    lines.push(`File: ${fileContent.path}`);
    if (fileContent.title) {
      lines.push(`Title: ${fileContent.title}`);
    }
    lines.push("");
    lines.push("Content:");
    lines.push("```");
    // Truncate very long content
    const content = fileContent.content.length > 8000
      ? fileContent.content.substring(0, 8000) + "\n... (truncated)"
      : fileContent.content;
    lines.push(content);
    lines.push("```");

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
      timeout: 120000,
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

  function handleInputFocus() {
    showDropdown = true;
  }

  function handleInputBlur() {
    // Delay to allow click on dropdown item
    setTimeout(() => {
      showDropdown = false;
    }, 200);
  }
</script>

<div class="summarizer-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <FileText size={16} />
      AI File Summary
    </h3>
  </div>

  <div class="panel-content">
    <!-- File Picker -->
    <div class="control-group">
      <label class="control-label">
        <Search size={14} />
        Select File
      </label>

      {#if selectedNote}
        <div class="selected-file">
          <span class="file-path">{selectedNote.title || selectedNote.path}</span>
          <button class="clear-btn" onclick={clearSelection} title="Clear selection">
            <X size={14} />
          </button>
        </div>
      {:else}
        <div class="file-picker">
          <input
            type="text"
            class="search-input"
            placeholder="Search for a file..."
            bind:value={searchQuery}
            onfocus={handleInputFocus}
            onblur={handleInputBlur}
          />

          {#if showDropdown && filteredNotes.length > 0}
            <div class="dropdown">
              {#each filteredNotes as note (note.id)}
                <button
                  class="dropdown-item"
                  onmousedown={() => selectNote(note)}
                >
                  <span class="note-title">{note.title || note.path}</span>
                  {#if note.title}
                    <span class="note-path">{note.path}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- File Preview -->
    {#if fileContent}
      <div class="control-group">
        <span class="control-label">File Preview</span>
        <div class="file-preview">
          <pre>{fileContent.content.substring(0, 500)}{fileContent.content.length > 500 ? '...' : ''}</pre>
        </div>
      </div>
    {/if}

    <!-- Generate Button -->
    <button
      class="generate-btn"
      onclick={generateSummary}
      disabled={isLoading || !fileContent || !settings}
    >
      {#if isLoading}
        <Loader2 size={16} class="spinning" />
        Generating...
      {:else}
        <Sparkles size={16} />
        Summarize File
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

  .file-picker {
    position: relative;
  }

  .search-input {
    width: 100%;
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    max-height: 200px;
    overflow-y: auto;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    z-index: 100;
  }

  .dropdown-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    width: 100%;
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .dropdown-item:hover {
    background: var(--bg-hover);
  }

  .note-title {
    color: var(--text-primary);
  }

  .note-path {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .selected-file {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
  }

  .file-path {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .clear-btn {
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .file-preview {
    max-height: 150px;
    overflow-y: auto;
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
  }

  .file-preview pre {
    margin: 0;
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
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
