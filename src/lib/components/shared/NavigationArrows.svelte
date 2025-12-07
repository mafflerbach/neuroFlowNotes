<script lang="ts">
  import { ChevronLeft, ChevronRight } from "lucide-svelte";
  import type { Snippet } from "svelte";

  interface Props {
    onPrevious: () => void;
    onNext: () => void;
    prevLabel?: string;
    nextLabel?: string;
    children?: Snippet;
  }

  let {
    onPrevious,
    onNext,
    prevLabel = "Previous",
    nextLabel = "Next",
    children,
  }: Props = $props();
</script>

<div class="navigation-arrows">
  <button class="nav-arrow" onclick={onPrevious} aria-label={prevLabel}>
    <ChevronLeft size={20} />
  </button>

  <div class="nav-content">
    {#if children}
      {@render children()}
    {/if}
  </div>

  <button class="nav-arrow" onclick={onNext} aria-label={nextLabel}>
    <ChevronRight size={20} />
  </button>
</div>

<style>
  .navigation-arrows {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-2);
  }

  .nav-content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nav-arrow {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: var(--radius-lg);
    color: var(--text-primary);
    cursor: pointer;
    flex-shrink: 0;
  }

  .nav-arrow:hover {
    background: var(--bg-hover);
  }
</style>
