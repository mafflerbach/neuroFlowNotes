<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    variant?: "normal" | "uppercase";
    badge?: string | number;
    actions?: Snippet;
  }

  let { title, variant = "normal", badge, actions }: Props = $props();
</script>

<div class="panel-header">
  <div class="header-left">
    <h3 class="panel-title" class:uppercase={variant === "uppercase"}>{title}</h3>
    {#if badge !== undefined}
      <span class="badge">{badge}</span>
    {/if}
  </div>

  <div class="header-actions">
    {#if actions}
      {@render actions()}
    {/if}
  </div>
</div>

<style>
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .panel-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .panel-title.uppercase {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .badge {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }
</style>
