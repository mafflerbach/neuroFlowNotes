<script lang="ts">
  import { FuzzySelect, MultiValueInput } from "../shared";
  import type {
    PropertyKeyInfo,
    PropertyFilter,
    PropertyOperator,
    FilterMatchMode,
  } from "../../types";
  import {
    VALUELESS_OPERATORS,
    MULTI_VALUE_OPERATORS,
    DATE_OPERATORS,
    isSpecialKey,
    getOperatorLabels,
    getPropertyType,
    buildPropertyKeyOptions,
    type SelectOption,
  } from "./queryUtils";

  interface Props {
    filters: PropertyFilter[];
    matchMode: FilterMatchMode;
    propertyKeys: PropertyKeyInfo[];
    propertyValues: Record<string, string[]>;
    allTags: string[];
    folderPaths: string[];
    loadingKeys: boolean;
    onUpdateFilters: (filters: PropertyFilter[]) => void;
    onUpdateMatchMode: (mode: FilterMatchMode) => void;
    onLoadValuesForKey: (key: string) => void;
  }

  let {
    filters,
    matchMode,
    propertyKeys,
    propertyValues,
    allTags,
    folderPaths,
    loadingKeys,
    onUpdateFilters,
    onUpdateMatchMode,
    onLoadValuesForKey,
  }: Props = $props();

  // Derived property key options
  const propertyKeyOptions = $derived(buildPropertyKeyOptions(propertyKeys));

  // Get value suggestions for a filter
  function getValueOptions(key: string): SelectOption[] {
    if (key === "_tags") {
      return allTags.map((tag) => ({ value: tag, label: tag }));
    }
    if (key === "_path") {
      return folderPaths.map((path) => ({ value: path, label: path }));
    }
    const values = propertyValues[key] || [];
    return values.map((v) => ({ value: v, label: v }));
  }

  // Add a new filter row
  function addFilter() {
    onUpdateFilters([...filters, { key: "", operator: "Exists", value: null }]);
  }

  // Remove a filter row
  function removeFilter(index: number) {
    if (filters.length <= 1) return;
    onUpdateFilters(filters.filter((_, i) => i !== index));
  }

  // Update a filter's key
  function updateFilterKey(index: number, key: string) {
    const newFilters = [...filters];
    newFilters[index] = { ...newFilters[index], key };

    // Reset operator to a valid one for this key's type
    const validOps = Object.keys(getOperatorLabels(key, propertyKeys));
    if (!validOps.includes(newFilters[index].operator)) {
      newFilters[index].operator = validOps[0] as PropertyOperator;
    }
    // Clear value when switching keys
    newFilters[index].value = null;

    onUpdateFilters(newFilters);

    if (key && !isSpecialKey(key)) {
      onLoadValuesForKey(key);
    }
  }

  // Update a filter's operator
  function updateFilterOperator(index: number, operator: PropertyOperator) {
    const newFilters = [...filters];
    newFilters[index] = { ...newFilters[index], operator };
    // Clear value if switching to valueless operator
    if (VALUELESS_OPERATORS.includes(operator)) {
      newFilters[index].value = null;
    }
    onUpdateFilters(newFilters);
  }

  // Update a filter's value
  function updateFilterValue(index: number, value: string) {
    const newFilters = [...filters];
    newFilters[index] = { ...newFilters[index], value: value || null };
    onUpdateFilters(newFilters);
  }
</script>

<div class="filters-section">
  <div class="filters-header">
    <span class="section-label">Filters</span>
    <div class="match-mode">
      <label>
        <input
          type="radio"
          name="matchMode"
          value="All"
          checked={matchMode === "All"}
          onchange={() => onUpdateMatchMode("All")}
        />
        All (AND)
      </label>
      <label>
        <input
          type="radio"
          name="matchMode"
          value="Any"
          checked={matchMode === "Any"}
          onchange={() => onUpdateMatchMode("Any")}
        />
        Any (OR)
      </label>
    </div>
  </div>

  {#if loadingKeys}
    <div class="loading-keys">Loading properties...</div>
  {:else}
    <div class="filter-rows">
      {#each filters as filter, index (index)}
        <div class="filter-row">
          <div class="key-select-wrapper">
            <FuzzySelect
              options={propertyKeyOptions}
              value={filter.key}
              onSelect={(key) => updateFilterKey(index, key)}
              placeholder="Select property..."
              emptyMessage="No properties found"
            />
          </div>

          <select
            class="operator-select"
            value={filter.operator}
            onchange={(e) =>
              updateFilterOperator(index, e.currentTarget.value as PropertyOperator)}
          >
            {#each Object.entries(getOperatorLabels(filter.key, propertyKeys)) as [op, label]}
              <option value={op}>{label}</option>
            {/each}
          </select>

          {#if !VALUELESS_OPERATORS.includes(filter.operator)}
            {@const valueOptions = getValueOptions(filter.key)}
            {@const isMultiValue = MULTI_VALUE_OPERATORS.includes(filter.operator)}
            {@const isDateOp = DATE_OPERATORS.includes(filter.operator)}
            {@const propType = getPropertyType(filter.key, propertyKeys)}
            {#if isDateOp || propType === "date"}
              <input
                type="date"
                class="value-input date-input"
                value={filter.value ?? ""}
                oninput={(e) => updateFilterValue(index, e.currentTarget.value)}
              />
            {:else if propType === "boolean"}
              <select
                class="value-input"
                value={filter.value ?? "true"}
                onchange={(e) => updateFilterValue(index, e.currentTarget.value)}
              >
                <option value="true">true</option>
                <option value="false">false</option>
              </select>
            {:else if propType === "number"}
              <input
                type="number"
                class="value-input"
                placeholder="0"
                value={filter.value ?? ""}
                oninput={(e) => updateFilterValue(index, e.currentTarget.value)}
              />
            {:else if isMultiValue && valueOptions.length > 0}
              <MultiValueInput
                options={valueOptions}
                value={filter.value ?? ""}
                onChange={(val) => updateFilterValue(index, val)}
                placeholder="Add values..."
              />
            {:else if valueOptions.length > 0}
              <div class="value-select-wrapper">
                <FuzzySelect
                  options={valueOptions}
                  value={filter.value ?? ""}
                  onSelect={(val) => updateFilterValue(index, val)}
                  placeholder="Select value..."
                  emptyMessage="No values found"
                />
              </div>
            {:else}
              <input
                type="text"
                class="value-input"
                placeholder={isMultiValue ? "Values (comma-separated)..." : "Value..."}
                value={filter.value ?? ""}
                oninput={(e) => updateFilterValue(index, e.currentTarget.value)}
              />
            {/if}
          {/if}

          <div class="filter-actions">
            <button class="add-btn" onclick={addFilter} title="Add filter">+</button>
            <button
              class="remove-btn"
              onclick={() => removeFilter(index)}
              disabled={filters.length <= 1}
              title="Remove filter"
            >-</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .filters-section {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .filters-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-3);
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .match-mode {
    display: flex;
    gap: var(--spacing-3);
  }

  .match-mode label {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-primary);
  }

  .match-mode input[type="radio"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  .loading-keys {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--spacing-2);
  }

  .filter-rows {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .filter-row {
    display: flex;
    gap: var(--spacing-2);
    align-items: flex-start;
  }

  .key-select-wrapper {
    flex: 1;
    min-width: 140px;
  }

  .value-select-wrapper {
    flex: 1;
    min-width: 120px;
  }

  .operator-select {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    min-width: 130px;
  }

  .value-input {
    flex: 1;
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .value-input.date-input {
    min-width: 140px;
    cursor: pointer;
  }

  .value-input.date-input::-webkit-calendar-picker-indicator {
    cursor: pointer;
    filter: var(--calendar-icon-filter, none);
  }

  .filter-actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .add-btn,
  .remove-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-size-base);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .add-btn:hover {
    background: var(--color-success);
    color: var(--color-white);
    border-color: var(--color-success);
  }

  .remove-btn:hover:not(:disabled) {
    background: var(--color-error);
    color: var(--color-white);
    border-color: var(--color-error);
  }

  .remove-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
