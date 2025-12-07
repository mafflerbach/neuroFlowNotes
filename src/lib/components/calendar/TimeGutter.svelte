<script lang="ts">
  interface Props {
    hours: number[];
    slotHeight?: number;
  }

  let { hours, slotHeight = 48 }: Props = $props();
</script>

<div class="time-gutter">
  {#each hours as hour}
    <div class="time-label" style:height="{slotHeight}px">
      {hour.toString().padStart(2, "0")}:00
    </div>
  {/each}
</div>

<style>
  .time-gutter {
    width: 60px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
  }

  .time-label {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding-right: var(--spacing-3);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    transform: translateY(-8px);
  }
</style>
