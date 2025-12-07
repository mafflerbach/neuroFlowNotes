<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    message: string;
    actionLabel?: string;
    onAction?: () => void;
    icon?: Snippet;
  }

  let { message, actionLabel, onAction, icon }: Props = $props();
</script>

<div class="empty-state">
  {#if icon}
    <div class="empty-icon">
      {@render icon()}
    </div>
  {/if}

  <p class="empty-message">{message}</p>

  {#if actionLabel && onAction}
    <button class="empty-action" onclick={onAction}>
      {actionLabel}
    </button>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-8) var(--spacing-6);
    text-align: center;
  }

  .empty-icon {
    color: var(--text-muted);
    margin-bottom: var(--spacing-3);
  }

  .empty-message {
    font-size: var(--font-size-base);
    color: var(--text-muted);
    margin: 0;
  }

  .empty-action {
    margin-top: var(--spacing-3);
    font-size: var(--font-size-sm);
    color: var(--color-primary);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .empty-action:hover {
    background: var(--bg-hover);
  }
</style>
