/**
 * Logger utility - provides consistent logging with tag-based filtering.
 * Logs are only output in development mode.
 */

const isDev = import.meta.env.DEV;

export const logger = {
  /**
   * Log a message (dev only)
   */
  log: (tag: string, message: string, ...args: unknown[]): void => {
    if (isDev) {
      console.log(`[${tag}]`, message, ...args);
    }
  },

  /**
   * Log an error (always, even in production)
   */
  error: (tag: string, message: string, ...args: unknown[]): void => {
    console.error(`[${tag}]`, message, ...args);
  },

  /**
   * Log a warning (dev only)
   */
  warn: (tag: string, message: string, ...args: unknown[]): void => {
    if (isDev) {
      console.warn(`[${tag}]`, message, ...args);
    }
  },

  /**
   * Log debug info (dev only)
   */
  debug: (tag: string, message: string, ...args: unknown[]): void => {
    if (isDev) {
      console.debug(`[${tag}]`, message, ...args);
    }
  },
};
