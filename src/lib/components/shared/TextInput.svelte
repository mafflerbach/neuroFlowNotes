<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLInputAttributes, "value"> {
    value?: string;
    variant?: "default" | "small" | "inline";
  }

  let {
    value = $bindable(""),
    variant = "default",
    type = "text",
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  class="text-input {variant} {className}"
  autocomplete="off"
  autocorrect="off"
  autocapitalize="off"
  spellcheck="false"
  {...rest}
/>

<style>
  .text-input {
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    font-family: inherit;
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    outline: none;
    transition: border-color var(--transition-normal), box-shadow var(--transition-normal);
  }

  .text-input:focus {
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .text-input::placeholder {
    color: var(--text-muted);
  }

  .text-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Variants */
  .text-input.small {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
  }

  .text-input.inline {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border-color: transparent;
    background: transparent;
  }

  .text-input.inline:hover {
    border-color: var(--input-border);
    background: var(--input-bg);
  }

  .text-input.inline:focus {
    border-color: var(--input-border-focus);
    background: var(--input-bg);
  }
</style>
