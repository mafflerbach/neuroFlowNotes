<script lang="ts">
  /**
   * DraggableScheduleBlock - A schedule block with drag & drop and resize support.
   * Used by CalendarDaily and CalendarWeekly for consistent block rendering.
   */
  import { Pencil } from "lucide-svelte";
  import type { ScheduleBlockDto } from "../../types";
  import { formatTimeShort } from "../../utils/blockLayoutUtils";
  import type { BlockConfig } from "../../utils/calendarShared.svelte";

  interface Props {
    block: ScheduleBlockDto;
    style: string;
    config?: BlockConfig;
    isDragging?: boolean;
    isResizing?: boolean;
    onBlockClick?: (block: ScheduleBlockDto) => void;
    onBlockEdit?: (block: ScheduleBlockDto) => void;
    onDragStart?: (e: DragEvent, block: ScheduleBlockDto) => void;
    onDragEnd?: () => void;
    onResizeStart?: (e: MouseEvent, block: ScheduleBlockDto, edge: "top" | "bottom") => void;
  }

  let {
    block,
    style,
    config = { size: "medium", showContext: false, showTime: true },
    isDragging = false,
    isResizing = false,
    onBlockClick,
    onBlockEdit,
    onDragStart,
    onDragEnd,
    onResizeStart,
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

  function handleDragStart(e: DragEvent) {
    onDragStart?.(e, block);
  }

  function handleResizeTop(e: MouseEvent) {
    onResizeStart?.(e, block, "top");
  }

  function handleResizeBottom(e: MouseEvent) {
    onResizeStart?.(e, block, "bottom");
  }
</script>

<div
  class="schedule-block size-{config.size}"
  class:is-dragging={isDragging}
  class:is-resizing={isResizing}
  {style}
  draggable="true"
  onclick={handleClick}
  onkeydown={handleKeydown}
  ondragstart={handleDragStart}
  ondragend={onDragEnd}
  role="button"
  tabindex="0"
  title={block.label || ""}
>
  <!-- Top resize handle -->
  {#if onResizeStart}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle resize-handle-top"
      onmousedown={handleResizeTop}
    ></div>
  {/if}

  <div class="block-content">
    {#if config.showTime}
      <span class="block-time">
        {formatTimeShort(block.start_time)} - {formatTimeShort(block.end_time)}
      </span>
    {/if}
    {#if block.label}
      <span class="block-label">{block.label}</span>
    {/if}
    {#if config.showContext && block.context}
      <span class="block-context">{block.context}</span>
    {/if}
  </div>

  {#if onBlockEdit}
    <button
      class="block-edit-btn"
      onclick={handleEdit}
      title="Edit block"
    >
      <Pencil size={config.size === "small" ? 10 : config.size === "medium" ? 12 : 14} />
    </button>
  {/if}

  <!-- Bottom resize handle -->
  {#if onResizeStart}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle resize-handle-bottom"
      onmousedown={handleResizeBottom}
    ></div>
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

  /* Size variants */
  .schedule-block.size-small {
    border-radius: var(--radius-sm);
    padding: var(--spacing-1);
    font-size: var(--font-size-xs);
  }

  .schedule-block.size-medium {
    border-radius: var(--radius-sm);
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-xs);
  }

  .schedule-block.size-large {
    border-radius: var(--radius-md);
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-md);
  }

  .schedule-block:hover {
    opacity: 1;
    box-shadow: var(--shadow-lg);
  }

  .schedule-block.is-dragging {
    opacity: 0.5;
    cursor: grabbing;
  }

  .schedule-block.is-resizing {
    opacity: 0.8;
    box-shadow: var(--shadow-lg);
  }

  .schedule-block:hover .block-edit-btn {
    opacity: 1;
  }

  /* Resize handles */
  .resize-handle {
    position: absolute;
    left: 0;
    right: 0;
    cursor: ns-resize;
    opacity: 0;
    transition: opacity var(--transition-normal);
  }

  .size-small .resize-handle,
  .size-medium .resize-handle {
    height: 6px;
  }

  .size-large .resize-handle {
    height: 8px;
  }

  .resize-handle-top {
    top: 0;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }

  .resize-handle-bottom {
    bottom: 0;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }

  .size-small .resize-handle-top,
  .size-medium .resize-handle-top {
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  }

  .size-small .resize-handle-bottom,
  .size-medium .resize-handle-bottom {
    border-radius: 0 0 var(--radius-sm) var(--radius-sm);
  }

  .schedule-block:hover .resize-handle {
    opacity: 1;
    background: rgba(0, 0, 0, 0.2);
  }

  .resize-handle:hover {
    background: rgba(0, 0, 0, 0.4) !important;
  }

  /* Block content */
  .block-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .block-time {
    font-weight: var(--font-weight-semibold);
  }

  .size-small .block-time,
  .size-medium .block-time {
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  .size-large .block-time {
    font-size: var(--font-size-sm);
  }

  .block-label {
    font-weight: var(--font-weight-medium);
  }

  .size-small .block-label,
  .size-medium .block-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .size-large .block-label {
    font-size: var(--font-size-md);
  }

  .block-context {
    font-size: var(--font-size-xs);
    opacity: 0.9;
  }

  /* Edit button */
  .block-edit-btn {
    position: absolute;
    top: var(--spacing-2);
    right: var(--spacing-2);
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

  .size-small .block-edit-btn {
    top: 2px;
    right: 2px;
    width: 16px;
    height: 16px;
  }

  .size-medium .block-edit-btn {
    top: 2px;
    right: 2px;
    width: 20px;
    height: 20px;
  }

  .size-large .block-edit-btn {
    width: 24px;
    height: 24px;
  }

  .block-edit-btn:hover {
    background: rgba(0, 0, 0, 0.4);
  }
</style>
