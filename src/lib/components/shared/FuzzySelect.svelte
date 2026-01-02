<script lang="ts">
  import { ChevronDown, Search } from "lucide-svelte";

  interface Option {
    value: string;
    label: string;
    group?: string;
    count?: number;
  }

  interface Props {
    options: Option[];
    value: string;
    onSelect: (value: string) => void;
    placeholder?: string;
    emptyMessage?: string;
  }

  let {
    options,
    value,
    onSelect,
    placeholder = "Select...",
    emptyMessage = "No options found",
  }: Props = $props();

  let query = $state("");
  let isOpen = $state(false);
  let highlightedIndex = $state(-1);
  let inputElement: HTMLInputElement | undefined = $state();
  let containerElement: HTMLDivElement | undefined = $state();

  // Simple fuzzy match function
  function fuzzyMatch(text: string, pattern: string): boolean {
    if (!pattern) return true;
    const lowerText = text.toLowerCase();
    const lowerPattern = pattern.toLowerCase();

    // Check for substring match first
    if (lowerText.includes(lowerPattern)) return true;

    // Simple fuzzy: all characters must appear in order
    let patternIdx = 0;
    for (let i = 0; i < lowerText.length && patternIdx < lowerPattern.length; i++) {
      if (lowerText[i] === lowerPattern[patternIdx]) {
        patternIdx++;
      }
    }
    return patternIdx === lowerPattern.length;
  }

  // Filter and sort options
  const filteredOptions = $derived.by(() => {
    let filtered = options.filter((opt) => fuzzyMatch(opt.label, query));

    // Sort alphabetically by label
    filtered.sort((a, b) => a.label.localeCompare(b.label));

    return filtered;
  });

  // Group filtered options
  const groupedOptions = $derived.by(() => {
    const groups: Map<string, Option[]> = new Map();

    for (const opt of filteredOptions) {
      const group = opt.group || "";
      if (!groups.has(group)) {
        groups.set(group, []);
      }
      groups.get(group)!.push(opt);
    }

    return groups;
  });

  // Flat list for keyboard navigation
  const flatOptions = $derived.by(() => {
    const result: Option[] = [];
    for (const [, opts] of groupedOptions) {
      result.push(...opts);
    }
    return result;
  });

  // Get display label for selected value
  const selectedLabel = $derived.by(() => {
    const opt = options.find((o) => o.value === value);
    return opt?.label || "";
  });

  function handleOpen() {
    isOpen = true;
    query = "";
    highlightedIndex = -1;
    // Focus the input after it appears
    setTimeout(() => inputElement?.focus(), 0);
  }

  function handleClose() {
    isOpen = false;
    query = "";
    highlightedIndex = -1;
  }

  function handleSelect(optValue: string) {
    onSelect(optValue);
    handleClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        highlightedIndex = Math.min(highlightedIndex + 1, flatOptions.length - 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        highlightedIndex = Math.max(highlightedIndex - 1, 0);
        break;
      case "Enter":
        e.preventDefault();
        if (highlightedIndex >= 0 && highlightedIndex < flatOptions.length) {
          handleSelect(flatOptions[highlightedIndex].value);
        }
        break;
      case "Escape":
        e.preventDefault();
        handleClose();
        break;
      case "Tab":
        handleClose();
        break;
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerElement && !containerElement.contains(e.target as Node)) {
      handleClose();
    }
  }

  // Track focus to close on blur
  $effect(() => {
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  });
</script>

<div class="fuzzy-select" bind:this={containerElement}>
  {#if isOpen}
    <div class="dropdown-container">
      <div class="search-input-wrapper">
        <Search size={14} />
        <input
          bind:this={inputElement}
          type="text"
          class="search-input"
          bind:value={query}
          onkeydown={handleKeydown}
          placeholder="Search..."
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
        />
      </div>
      <div class="dropdown-list">
        {#if flatOptions.length === 0}
          <div class="empty-message">{emptyMessage}</div>
        {:else}
          {#each [...groupedOptions] as [group, opts]}
            {#if group}
              <div class="group-header">{group}</div>
            {/if}
            {#each opts as opt (opt.value)}
              {@const optIndex = flatOptions.indexOf(opt)}
              <button
                type="button"
                class="dropdown-item"
                class:highlighted={optIndex === highlightedIndex}
                class:selected={opt.value === value}
                onmousedown={() => handleSelect(opt.value)}
                onmouseenter={() => (highlightedIndex = optIndex)}
              >
                <span class="item-label">{opt.label}</span>
                {#if opt.count !== undefined}
                  <span class="item-count">({opt.count})</span>
                {/if}
              </button>
            {/each}
          {/each}
        {/if}
      </div>
    </div>
  {:else}
    <button type="button" class="select-trigger" onclick={handleOpen}>
      <span class="trigger-text" class:placeholder={!value}>
        {value ? selectedLabel : placeholder}
      </span>
      <ChevronDown size={14} />
    </button>
  {/if}
</div>

<style>
  .fuzzy-select {
    position: relative;
    min-width: 120px;
  }

  .select-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-1);
    width: 100%;
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .select-trigger:hover {
    border-color: var(--border-default);
  }

  .trigger-text {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .trigger-text.placeholder {
    color: var(--text-muted);
  }

  .dropdown-container {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    background: var(--bg-surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    min-width: 200px;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    border-bottom: 1px solid var(--border-light);
    color: var(--text-muted);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .dropdown-list {
    max-height: 250px;
    overflow-y: auto;
  }

  .group-header {
    padding: var(--spacing-2) var(--spacing-2) var(--spacing-1);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--spacing-2);
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    text-align: left;
  }

  .dropdown-item:hover,
  .dropdown-item.highlighted {
    background: var(--bg-hover);
  }

  .dropdown-item.selected {
    background: var(--bg-selected);
    font-weight: var(--font-weight-medium);
  }

  .item-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-count {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    flex-shrink: 0;
    margin-left: var(--spacing-1);
  }

  .empty-message {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
</style>
