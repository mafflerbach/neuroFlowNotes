<!--
  Toast Component - Displays toast notifications from the notifications store.
  Place this once in your app layout (e.g., App.svelte).
-->
<script lang="ts">
  import { notifications, type Notification } from "../../stores/notifications.svelte";

  function getIcon(type: Notification["type"]): string {
    switch (type) {
      case "success":
        return "✓";
      case "error":
        return "✕";
      case "warning":
        return "⚠";
      case "info":
      default:
        return "ℹ";
    }
  }
</script>

{#if notifications.notifications.length > 0}
  <div class="toast-container">
    {#each notifications.notifications as notification (notification.id)}
      <div class="toast toast-{notification.type}">
        <span class="toast-icon">{getIcon(notification.type)}</span>
        <span class="toast-message">{notification.message}</span>
        <button
          class="toast-dismiss"
          onclick={() => notifications.dismiss(notification.id)}
          aria-label="Dismiss"
        >
          ✕
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: var(--spacing-4);
    right: var(--spacing-4);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-3) var(--spacing-4);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    animation: slideIn 0.2s ease-out;
    font-size: var(--font-size-sm);
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .toast-success {
    background: var(--color-success);
    color: var(--color-white);
  }

  .toast-error {
    background: var(--color-error);
    color: var(--color-white);
  }

  .toast-warning {
    background: var(--color-warning);
    color: var(--color-black);
  }

  .toast-info {
    background: var(--color-info);
    color: var(--color-white);
  }

  .toast-icon {
    flex-shrink: 0;
    font-size: var(--font-size-base);
  }

  .toast-message {
    flex: 1;
    line-height: 1.4;
  }

  .toast-dismiss {
    flex-shrink: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: var(--spacing-1);
    opacity: 0.7;
    transition: opacity 0.15s;
    font-size: var(--font-size-sm);
  }

  .toast-dismiss:hover {
    opacity: 1;
  }
</style>
