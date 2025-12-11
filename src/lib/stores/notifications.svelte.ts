/**
 * Notifications Store - Centralized toast notification management.
 * Provides a simple API for showing success/error/info messages across the app.
 */

export type NotificationType = "success" | "error" | "info" | "warning";

export interface Notification {
  id: number;
  type: NotificationType;
  message: string;
  duration: number;
}

const DEFAULT_DURATION = 3000;

class NotificationsStore {
  notifications = $state<Notification[]>([]);
  private nextId = 0;

  /**
   * Show a notification message.
   */
  show(type: NotificationType, message: string, duration = DEFAULT_DURATION): number {
    const id = this.nextId++;
    const notification: Notification = { id, type, message, duration };

    this.notifications = [...this.notifications, notification];

    if (duration > 0) {
      setTimeout(() => this.dismiss(id), duration);
    }

    return id;
  }

  /**
   * Show a success notification.
   */
  success(message: string, duration = DEFAULT_DURATION): number {
    return this.show("success", message, duration);
  }

  /**
   * Show an error notification.
   */
  error(message: string, duration = DEFAULT_DURATION): number {
    return this.show("error", message, duration);
  }

  /**
   * Show an info notification.
   */
  info(message: string, duration = DEFAULT_DURATION): number {
    return this.show("info", message, duration);
  }

  /**
   * Show a warning notification.
   */
  warning(message: string, duration = DEFAULT_DURATION): number {
    return this.show("warning", message, duration);
  }

  /**
   * Dismiss a notification by ID.
   */
  dismiss(id: number): void {
    this.notifications = this.notifications.filter(n => n.id !== id);
  }

  /**
   * Dismiss all notifications.
   */
  dismissAll(): void {
    this.notifications = [];
  }
}

export const notifications = new NotificationsStore();
