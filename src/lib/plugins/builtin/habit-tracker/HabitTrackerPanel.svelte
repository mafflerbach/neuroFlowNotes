<script lang="ts">
  /**
   * Habit Tracker Panel
   *
   * Sidebar panel for managing habits and creating embeds.
   */
  import { pluginRegistry } from "../../registry.svelte";
  import type { HabitTrackerSettings } from "./types";
  import type {
    HabitDto,
    CreateHabitRequest,
    HabitType,
    HabitViewType,
    HabitDateRange,
    HabitTableOrientation,
  } from "../../../types";
  import {
    listHabits,
    createHabit,
    toggleHabit,
    archiveHabit,
  } from "../../../services/api";
  import {
    CheckCircle,
    Plus,
    Code,
    Copy,
    Check,
    X,
    Trash2,
    RefreshCw,
  } from "lucide-svelte";
  import { TextInput } from "$lib/components/shared";

  // Get plugin settings
  const plugin = $derived(
    pluginRegistry.get<HabitTrackerSettings>("habit-tracker")
  );
  const settings = $derived(plugin?.config.settings);

  // State
  let habits = $state<HabitDto[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let today = $state(new Date().toISOString().split("T")[0]);

  // Create habit form state
  let showCreateForm = $state(false);
  let newHabitName = $state("");
  let newHabitType = $state<HabitType>("boolean");
  let newHabitUnit = $state("");
  let newHabitTarget = $state<number | null>(null);
  let creating = $state(false);

  // Embed builder state
  let showEmbedBuilder = $state(false);
  let selectedHabits = $state<string[]>([]);
  let embedView = $state<HabitViewType>("table");
  let embedOrientation = $state<HabitTableOrientation>("horizontal");
  let embedDateRange = $state<HabitDateRange>("last7_days");
  let embedDate = $state("");
  let embedEditable = $state(true);
  let embedShowSummary = $state(true);
  let isCopied = $state(false);

  // Track toggle states for today's habits
  let habitToggles = $state<Record<number, boolean>>({});

  // Load habits on mount
  $effect(() => {
    loadHabits();
  });

  // Set defaults from settings
  $effect(() => {
    if (settings) {
      embedView = settings.defaultView as HabitViewType;
      embedDateRange = settings.defaultDateRange as HabitDateRange;
    }
  });

  async function loadHabits() {
    loading = true;
    error = null;
    try {
      habits = await listHabits(false);
    } catch (e) {
      console.error("Failed to load habits:", e);
      error = "Failed to load habits";
    } finally {
      loading = false;
    }
  }

  async function handleToggle(habit: HabitDto) {
    if (habit.habit_type !== "boolean") return;

    try {
      const isDone = await toggleHabit(habit.id, today);
      habitToggles[habit.id] = isDone;
    } catch (e) {
      console.error("Failed to toggle habit:", e);
    }
  }

  async function handleCreateHabit() {
    if (!newHabitName.trim()) return;

    creating = true;
    error = null;

    try {
      const request: CreateHabitRequest = {
        name: newHabitName.trim(),
        habit_type: newHabitType,
        unit: newHabitType === "number" && newHabitUnit ? newHabitUnit : null,
        target_value: newHabitType === "number" ? newHabitTarget : null,
        color: settings?.defaultColor ?? "#6366f1",
      };

      await createHabit(request);
      await loadHabits();

      // Reset form
      newHabitName = "";
      newHabitType = "boolean";
      newHabitUnit = "";
      newHabitTarget = null;
      showCreateForm = false;
    } catch (e) {
      console.error("Failed to create habit:", e);
      error = e instanceof Error ? e.message : "Failed to create habit";
    } finally {
      creating = false;
    }
  }

  async function handleArchiveHabit(habit: HabitDto) {
    try {
      await archiveHabit(habit.id);
      await loadHabits();
    } catch (e) {
      console.error("Failed to archive habit:", e);
    }
  }

  function toggleHabitSelection(habitName: string) {
    if (selectedHabits.includes(habitName)) {
      selectedHabits = selectedHabits.filter((h) => h !== habitName);
    } else {
      selectedHabits = [...selectedHabits, habitName];
    }
  }

  function generateEmbedCode(): string {
    const lines: string[] = ["```habit-tracker"];

    if (selectedHabits.length > 0) {
      lines.push("habits:");
      for (const habit of selectedHabits) {
        lines.push(`  - "${habit}"`);
      }
    }

    lines.push(`view: ${embedView}`);

    if (embedView === "table" && embedOrientation !== "horizontal") {
      lines.push(`orientation: ${embedOrientation}`);
    }

    lines.push(`date_range: ${embedDateRange}`);

    if (embedDate.trim()) {
      lines.push(`date: ${embedDate.trim()}`);
    }

    if (!embedEditable) {
      lines.push("editable: false");
    }

    if (!embedShowSummary) {
      lines.push("show_summary: false");
    }

    lines.push("```");

    return lines.join("\n");
  }

  async function copyEmbedCode() {
    try {
      await navigator.clipboard.writeText(generateEmbedCode());
      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  const habitTypeLabels: Record<HabitType, string> = {
    boolean: "Yes/No",
    number: "Number",
    text: "Text",
    rating: "Rating",
  };

  const viewTypeLabels: Record<HabitViewType, string> = {
    table: "Table",
    calendar: "Calendar",
    streak: "Streak",
    list: "List",
  };

  const orientationLabels: Record<HabitTableOrientation, string> = {
    horizontal: "Habits as rows",
    vertical: "Dates as rows",
  };

  const dateRangeLabels: Record<HabitDateRange, string> = {
    single_day: "Single Day",
    last7_days: "Last 7 Days",
    last30_days: "Last 30 Days",
    this_week: "This Week",
    this_month: "This Month",
    custom: "Custom",
  };
</script>

<div class="habit-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <CheckCircle size={16} />
      Habit Tracker
    </h3>
    <div class="header-actions">
      <button
        class="icon-btn"
        onclick={loadHabits}
        title="Refresh"
        disabled={loading}
      >
        <RefreshCw size={14} />
      </button>
      <button
        class="icon-btn"
        onclick={() => (showCreateForm = !showCreateForm)}
        title="New habit"
      >
        <Plus size={14} />
      </button>
    </div>
  </div>

  <div class="panel-content">
    {#if error}
      <div class="error-message">{error}</div>
    {/if}

    <!-- Create Habit Form -->
    {#if showCreateForm}
      <div class="create-form">
        <div class="form-header">
          <span class="form-title">New Habit</span>
          <button class="icon-btn" onclick={() => (showCreateForm = false)}>
            <X size={14} />
          </button>
        </div>

        <div class="form-field">
          <label for="habit-name">Name</label>
          <TextInput
            id="habit-name"
            class="text-input"
            bind:value={newHabitName}
            placeholder="e.g., Drink Water"
          />
        </div>

        <div class="form-field">
          <label for="habit-type">Type</label>
          <select id="habit-type" class="select-input" bind:value={newHabitType}>
            {#each Object.entries(habitTypeLabels) as [value, label]}
              <option {value}>{label}</option>
            {/each}
          </select>
        </div>

        {#if newHabitType === "number"}
          <div class="form-row">
            <div class="form-field">
              <label for="habit-unit">Unit</label>
              <TextInput
                id="habit-unit"
                class="text-input"
                bind:value={newHabitUnit}
                placeholder="e.g., glasses"
              />
            </div>
            <div class="form-field">
              <label for="habit-target">Target</label>
              <input
                type="number"
                id="habit-target"
                class="text-input"
                bind:value={newHabitTarget}
                placeholder="8"
              />
            </div>
          </div>
        {/if}

        <button
          class="primary-btn"
          onclick={handleCreateHabit}
          disabled={!newHabitName.trim() || creating}
        >
          {creating ? "Creating..." : "Create Habit"}
        </button>
      </div>
    {/if}

    <!-- Habits List -->
    <div class="habits-section">
      <div class="section-header">
        <span class="section-title">Today's Habits</span>
        <span class="section-date">{today}</span>
      </div>

      {#if loading}
        <div class="loading">Loading habits...</div>
      {:else if habits.length === 0}
        <div class="empty-state">
          No habits yet. Click + to create one.
        </div>
      {:else}
        <div class="habits-list">
          {#each habits as habit}
            <div class="habit-item">
              {#if habit.habit_type === "boolean"}
                <button
                  class="toggle-btn"
                  class:checked={habitToggles[habit.id]}
                  onclick={() => handleToggle(habit)}
                >
                  {#if habitToggles[habit.id]}
                    <Check size={14} />
                  {/if}
                </button>
              {:else}
                <div
                  class="habit-type-badge"
                  style:background-color={habit.color ?? "#6366f1"}
                >
                  {habit.habit_type.charAt(0).toUpperCase()}
                </div>
              {/if}

              <div class="habit-info">
                <span class="habit-name">{habit.name}</span>
                {#if habit.habit_type === "number" && habit.target_value}
                  <span class="habit-target"
                    >Target: {habit.target_value}{habit.unit
                      ? ` ${habit.unit}`
                      : ""}</span
                  >
                {/if}
              </div>

              <button
                class="icon-btn danger"
                onclick={() => handleArchiveHabit(habit)}
                title="Archive habit"
              >
                <Trash2 size={12} />
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Embed Builder -->
    <div class="embed-section">
      <button
        class="section-toggle"
        onclick={() => (showEmbedBuilder = !showEmbedBuilder)}
      >
        <Code size={14} />
        <span>Embed Builder</span>
        <span class="toggle-indicator">{showEmbedBuilder ? "−" : "+"}</span>
      </button>

      {#if showEmbedBuilder}
        <div class="embed-builder">
          <!-- Habit Selection -->
          <div class="form-field">
            <label>Select Habits</label>
            <div class="habit-checkboxes">
              {#each habits as habit}
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    checked={selectedHabits.includes(habit.name)}
                    onchange={() => toggleHabitSelection(habit.name)}
                  />
                  <span>{habit.name}</span>
                </label>
              {/each}
              {#if habits.length === 0}
                <span class="hint">No habits available</span>
              {/if}
            </div>
          </div>

          <!-- View Type -->
          <div class="form-field">
            <label for="embed-view">View</label>
            <select id="embed-view" class="select-input" bind:value={embedView}>
              {#each Object.entries(viewTypeLabels) as [value, label]}
                <option {value}>{label}</option>
              {/each}
            </select>
          </div>

          <!-- Orientation (only for table view) -->
          {#if embedView === "table"}
            <div class="form-field">
              <label for="embed-orientation">Orientation</label>
              <select id="embed-orientation" class="select-input" bind:value={embedOrientation}>
                {#each Object.entries(orientationLabels) as [value, label]}
                  <option {value}>{label}</option>
                {/each}
              </select>
            </div>
          {/if}

          <!-- Date Range -->
          <div class="form-field">
            <label for="embed-range">Date Range</label>
            <select
              id="embed-range"
              class="select-input"
              bind:value={embedDateRange}
            >
              {#each Object.entries(dateRangeLabels).filter(([k]) => k !== "custom") as [value, label]}
                <option {value}>{label}</option>
              {/each}
            </select>
          </div>

          <!-- Reference Date -->
          <div class="form-field">
            <label for="embed-date">Reference Date</label>
            <TextInput
              id="embed-date"
              class="text-input"
              bind:value={embedDate}
              placeholder="2025-01-15 or template var"
            />
            <span class="hint">Leave empty for today, or use template variable</span>
          </div>

          <!-- Options -->
          <div class="form-options">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={embedEditable} />
              <span>Editable</span>
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={embedShowSummary} />
              <span>Show Summary</span>
            </label>
          </div>

          <!-- Code Preview -->
          <div class="code-preview">
            <pre>{generateEmbedCode()}</pre>
          </div>

          <!-- Copy Button -->
          <button class="copy-btn" onclick={copyEmbedCode}>
            {#if isCopied}
              <Check size={14} />
              Copied!
            {:else}
              <Copy size={14} />
              Copy Embed Code
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .habit-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .header-actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .icon-btn.danger:hover {
    color: var(--color-error);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-3);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .error-message {
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    color: var(--color-error);
    background: var(--color-error-bg);
    border-radius: var(--radius-sm);
  }

  /* Create Form */
  .create-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-light);
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .form-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .form-field label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-2);
  }

  .text-input,
  .select-input {
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .text-input:focus,
  .select-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .primary-btn {
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--btn-primary-text);
    background: var(--color-primary);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .primary-btn:hover:not(:disabled) {
    background: var(--btn-primary-bg-hover);
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Habits Section */
  .habits-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .section-date {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .loading,
  .empty-state {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-style: italic;
    text-align: center;
    padding: var(--spacing-4);
  }

  .habits-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .habit-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-light);
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: 2px solid var(--border-medium);
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .toggle-btn:hover {
    border-color: var(--color-primary);
  }

  .toggle-btn.checked {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .habit-type-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: white;
  }

  .habit-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .habit-name {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .habit-target {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  /* Embed Section */
  .embed-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    border-top: 1px solid var(--border-light);
    padding-top: var(--spacing-3);
  }

  .section-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    width: 100%;
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    background: var(--bg-surface-sunken);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .section-toggle:hover {
    background: var(--bg-hover);
  }

  .toggle-indicator {
    margin-left: auto;
    color: var(--text-muted);
  }

  .embed-builder {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-sm);
  }

  .habit-checkboxes {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
    max-height: 120px;
    overflow-y: auto;
    padding: var(--spacing-2);
    background: var(--bg-surface);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    cursor: pointer;
  }

  .checkbox-label input {
    cursor: pointer;
  }

  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-style: italic;
  }

  .form-options {
    display: flex;
    gap: var(--spacing-3);
  }

  .code-preview {
    padding: var(--spacing-2);
    background: var(--bg-surface);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    overflow-x: auto;
  }

  .code-preview pre {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    white-space: pre;
  }

  .copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    background: var(--bg-surface);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .copy-btn:hover {
    background: var(--bg-hover);
    border-color: var(--color-primary);
  }
</style>
