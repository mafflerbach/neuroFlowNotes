<script lang="ts">
  interface Props {
    value: string;
    placeholder?: string;
    selectOnMount?: boolean;
    onConfirm: (value: string) => void;
    onCancel: () => void;
  }

  let {
    value = "",
    placeholder = "",
    selectOnMount = true,
    onConfirm,
    onCancel,
  }: Props = $props();

  let inputValue = $state(value);

  function handleMount(node: HTMLInputElement) {
    if (selectOnMount) {
      node.focus();
      node.select();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      const trimmed = inputValue.trim();
      if (trimmed) {
        onConfirm(trimmed);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    }
  }

  function handleBlur() {
    const trimmed = inputValue.trim();
    if (trimmed && trimmed !== value) {
      onConfirm(trimmed);
    } else {
      onCancel();
    }
  }
</script>

<input
  type="text"
  class="inline-input"
  use:handleMount
  bind:value={inputValue}
  {placeholder}
  onkeydown={handleKeydown}
  onblur={handleBlur}
/>

<style>
  .inline-input {
    width: 100%;
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-base);
    border: 1px solid var(--input-border-focus);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    outline: none;
  }

  .inline-input:focus {
    box-shadow: var(--shadow-focus);
  }
</style>
