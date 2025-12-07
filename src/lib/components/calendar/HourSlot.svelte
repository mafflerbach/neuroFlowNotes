<script lang="ts">
  interface Props {
    hour: number;
    height?: number;
    onClick?: (hour: number) => void;
  }

  let { hour, height = 48, onClick }: Props = $props();

  function handleClick() {
    onClick?.(hour);
  }
</script>

<button
  class="hour-slot"
  style:height="{height}px"
  onclick={handleClick}
>
  <span class="slot-time">{hour}:00</span>
</button>

<style>
  .hour-slot {
    width: 100%;
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--border-light);
    cursor: pointer;
    text-align: left;
    padding: var(--spacing-1) var(--spacing-2);
  }

  .hour-slot:hover {
    background: var(--bg-hover);
  }

  .slot-time {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    opacity: 0;
  }

  .hour-slot:hover .slot-time {
    opacity: 1;
  }
</style>
