<script lang="ts">
  import { X } from "lucide-svelte";
  import type { Snippet } from "svelte";

  interface Props {
    open: boolean;
    title: string;
    maxWidth?: string;
    onClose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  let { open, title, maxWidth = "520px", onClose, children, footer }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="modal" style="max-width: {maxWidth}">
      <div class="modal-header">
        <h2 id="modal-title" class="modal-title">{title}</h2>
        <button class="close-btn" onclick={onClose} aria-label="Close">
          <X size={18} />
        </button>
      </div>

      <div class="modal-body">
        {@render children()}
      </div>

      {#if footer}
        <div class="modal-footer">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: var(--modal-backdrop-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal-backdrop);
  }

  .modal {
    background: var(--modal-bg);
    border-radius: var(--radius-xl);
    box-shadow: var(--modal-shadow);
    width: 90%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    z-index: var(--z-modal);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-4) var(--spacing-5);
    border-bottom: 1px solid var(--modal-border);
  }

  .modal-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-5);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-3);
    padding: var(--spacing-4) var(--spacing-5);
    border-top: 1px solid var(--modal-border);
  }
</style>
