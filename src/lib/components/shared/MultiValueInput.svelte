<script lang="ts">
  import { X } from "lucide-svelte";

  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    options: Option[];
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
  }

  let {
    options,
    value,
    onChange,
    placeholder = "Add value...",
  }: Props = $props();

  let query = $state("");
  let isOpen = $state(false);
  let highlightedIndex = $state(-1);
  let inputElement: HTMLInputElement | undefined = $state();
  let containerElement: HTMLDivElement | undefined = $state();

  // Parse current values from comma-separated string
  const selectedValues = $derived.by(() => {
    if (!value) return [];
    return value.split(",").map((v) => v.trim()).filter((v) => v);
  });

  // Filter options based on query and exclude already selected
  const filteredOptions = $derived.by(() => {
    const selected = new Set(selectedValues);
    let filtered = options.filter((opt) => !selected.has(opt.value));

    if (query.trim()) {
      const q = query.toLowerCase();
      filtered = filtered.filter((opt) =>
        opt.label.toLowerCase().includes(q) || opt.value.toLowerCase().includes(q)
      );
    }

    return filtered.slice(0, 20);
  });

  function addValue(val: string) {
    const current = selectedValues;
    if (!current.includes(val)) {
      const newValues = [...current, val];
      onChange(newValues.join(", "));
    }
    query = "";
    highlightedIndex = -1;
    inputElement?.focus();
  }

  function removeValue(val: string) {
    const newValues = selectedValues.filter((v) => v !== val);
    onChange(newValues.join(", "));
  }

  function handleInputFocus() {
    isOpen = true;
  }

  function handleInputBlur() {
    setTimeout(() => {
      isOpen = false;
      highlightedIndex = -1;
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        if (!isOpen) isOpen = true;
        highlightedIndex = Math.min(highlightedIndex + 1, filteredOptions.length - 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        highlightedIndex = Math.max(highlightedIndex - 1, 0);
        break;
      case "Enter":
        e.preventDefault();
        if (highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
          addValue(filteredOptions[highlightedIndex].value);
        } else if (query.trim()) {
          // Allow adding custom value
          addValue(query.trim());
        }
        break;
      case "Escape":
        e.preventDefault();
        isOpen = false;
        highlightedIndex = -1;
        break;
      case "Backspace":
        if (!query && selectedValues.length > 0) {
          removeValue(selectedValues[selectedValues.length - 1]);
        }
        break;
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerElement && !containerElement.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  });
</script>

<div class="multi-value-input" bind:this={containerElement}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="input-container" onclick={() => inputElement?.focus()}>
    {#each selectedValues as val}
      <span class="value-tag">
        {val}
        <button
          type="button"
          class="remove-tag"
          onmousedown={(e) => { e.preventDefault(); removeValue(val); }}
        >
          <X size={12} />
        </button>
      </span>
    {/each}
    <input
      bind:this={inputElement}
      type="text"
      class="tag-input"
      bind:value={query}
      {placeholder}
      onfocus={handleInputFocus}
      onblur={handleInputBlur}
      onkeydown={handleKeydown}
    />
  </div>

  {#if isOpen && filteredOptions.length > 0}
    <div class="dropdown">
      {#each filteredOptions as opt, i (opt.value)}
        <button
          type="button"
          class="dropdown-item"
          class:highlighted={i === highlightedIndex}
          onmousedown={() => addValue(opt.value)}
          onmouseenter={() => (highlightedIndex = i)}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .multi-value-input {
    position: relative;
    flex: 1;
    min-width: 120px;
  }

  .input-container {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-1);
    padding: var(--spacing-1);
    min-height: 28px;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    cursor: text;
  }

  .input-container:focus-within {
    border-color: var(--border-default);
  }

  .value-tag {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 1px 4px 1px 6px;
    background: var(--bg-surface-raised);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }

  .remove-tag {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    width: 14px;
    height: 14px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .remove-tag:hover {
    background: var(--color-error);
    color: var(--color-white);
  }

  .tag-input {
    flex: 1;
    min-width: 60px;
    padding: 0 var(--spacing-1);
    border: none;
    background: transparent;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    outline: none;
  }

  .tag-input::placeholder {
    color: var(--text-muted);
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--bg-surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    z-index: 100;
    max-height: 200px;
    overflow-y: auto;
  }

  .dropdown-item {
    display: block;
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
</style>
