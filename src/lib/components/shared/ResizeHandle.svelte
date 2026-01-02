<script lang="ts">
  interface Props {
    /** Direction of resize - 'horizontal' resizes width, 'vertical' resizes height */
    direction?: "horizontal" | "vertical";
    /** Position of the handle - determines which side it appears on */
    position?: "left" | "right" | "top" | "bottom";
    /** Callback when resizing with the delta in pixels */
    onResize: (delta: number) => void;
    /** Optional callback when resize starts */
    onResizeStart?: () => void;
    /** Optional callback when resize ends */
    onResizeEnd?: () => void;
  }

  let {
    direction = "horizontal",
    position = "right",
    onResize,
    onResizeStart,
    onResizeEnd,
  }: Props = $props();

  let isResizing = $state(false);
  let startPos = $state(0);

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    startPos = direction === "horizontal" ? e.clientX : e.clientY;
    onResizeStart?.();

    // Add listeners to window to capture mouse movement even outside the handle
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    // Prevent text selection during resize
    document.body.style.userSelect = "none";
    document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;

    const currentPos = direction === "horizontal" ? e.clientX : e.clientY;
    let delta = currentPos - startPos;

    // Invert delta for left/top handles since they resize in opposite direction
    if (position === "left" || position === "top") {
      delta = -delta;
    }

    onResize(delta);
    startPos = currentPos;
  }

  function handleMouseUp() {
    isResizing = false;
    onResizeEnd?.();

    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    document.body.style.userSelect = "";
    document.body.style.cursor = "";
  }
</script>

<div
  class="resize-handle"
  class:horizontal={direction === "horizontal"}
  class:vertical={direction === "vertical"}
  class:left={position === "left"}
  class:right={position === "right"}
  class:top={position === "top"}
  class:bottom={position === "bottom"}
  class:resizing={isResizing}
  onmousedown={handleMouseDown}
  role="separator"
  aria-orientation={direction}
  tabindex="0"
></div>

<style>
  .resize-handle {
    position: absolute;
    z-index: 10;
    background: transparent;
    transition: background var(--transition-fast);
  }

  /* Horizontal resize (left/right edge) */
  .resize-handle.horizontal {
    width: 6px;
    top: 0;
    bottom: 0;
    cursor: col-resize;
  }

  .resize-handle.horizontal.left {
    left: 0;
  }

  .resize-handle.horizontal.right {
    right: 0;
  }

  /* Vertical resize (top/bottom edge) */
  .resize-handle.vertical {
    height: 6px;
    left: 0;
    right: 0;
    cursor: row-resize;
  }

  .resize-handle.vertical.top {
    top: 0;
  }

  .resize-handle.vertical.bottom {
    bottom: 0;
  }

  /* Hover and active states */
  .resize-handle:hover,
  .resize-handle.resizing {
    background: var(--color-primary);
    opacity: 0.5;
  }

  .resize-handle.resizing {
    opacity: 0.7;
  }
</style>
