<script lang="ts">
  import { Pencil } from "lucide-svelte";
  import type { ScheduleBlockDto } from "../../types";
  import { formatTimeShort } from "../../utils/blockLayoutUtils";

  interface Props {
    block: ScheduleBlockDto;
    style: string;
    showTime?: boolean;
    showContext?: boolean;
    onBlockClick?: (block: ScheduleBlockDto) => void;
    onBlockEdit?: (block: ScheduleBlockDto) => void;
  }

  let {
    block,
    style,
    showTime = true,
    showContext = true,
    onBlockClick,
    onBlockEdit,
  }: Props = $props();

  function handleClick() {
    onBlockClick?.(block);
  }

  function handleEdit(e: MouseEvent) {
    e.stopPropagation();
    onBlockEdit?.(block);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      onBlockClick?.(block);
    }
  }
</script>

<div
  class="schedule-block"
  {style}
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="button"
  tabindex="0"
>
  <div class="block-content">
    {#if showTime}
      <span class="block-time">
        {formatTimeShort(block.start_time)} - {formatTimeShort(block.end_time)}
      </span>
    {/if}
    {#if block.label}
      <span class="block-label">{block.label}</span>
    {/if}
    {#if showContext && block.context}
      <span class="block-context">{block.context}</span>
    {/if}
  </div>
  {#if onBlockEdit}
    <button
      class="block-edit-btn"
      onclick={handleEdit}
      title="Edit block"
    >
      <Pencil size={14} />
    </button>
  {/if}
</div>

<style>
  .schedule-block {
    position: absolute;
    border: none;
    border-radius: var(--radius-md);
    padding: var(--spacing-2) var(--spacing-3);
    color: var(--block-default-text);
    cursor: pointer;
    text-align: left;
    pointer-events: auto;
    opacity: 0.95;
    overflow: hidden;
    box-sizing: border-box;
  }

  .schedule-block:hover {
    opacity: 1;
    box-shadow: var(--shadow-lg);
  }

  .schedule-block:hover .block-edit-btn {
    opacity: 1;
  }

  .block-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .block-time {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
  }

  .block-label {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
  }

  .block-context {
    font-size: var(--font-size-xs);
    opacity: 0.9;
  }

  .block-edit-btn {
    position: absolute;
    top: var(--spacing-2);
    right: var(--spacing-2);
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-normal), background var(--transition-normal);
    color: inherit;
  }

  .block-edit-btn:hover {
    background: rgba(0, 0, 0, 0.4);
  }
</style>
