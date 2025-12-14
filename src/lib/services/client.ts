/**
 * API Client - wrapper around Tauri invoke with error handling and logging.
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";

/**
 * Custom error class for API errors.
 */
export class ApiError extends Error {
  constructor(
    public command: string,
    public cause: unknown
  ) {
    const message = cause instanceof Error ? cause.message : String(cause);
    super(`API call failed: ${command} - ${message}`);
    this.name = "ApiError";
  }
}

/**
 * Options for API calls.
 */
export interface ApiCallOptions {
  /** Whether to log the call (default: true in development). */
  log?: boolean;
  /** Timeout in milliseconds (not currently enforced by Tauri). */
  timeout?: number;
}

/**
 * Wrapper around Tauri invoke with error handling and logging.
 *
 * @param command - The Tauri command name.
 * @param params - Optional parameters to pass to the command.
 * @param options - Optional configuration for the call.
 * @returns The result from the command.
 * @throws ApiError if the command fails.
 *
 * @example
 * ```ts
 * const notes = await apiCall<NoteListItem[]>("list_notes");
 * const note = await apiCall<NoteDto>("get_note", { noteId: 123 });
 * ```
 */
export async function apiCall<T>(
  command: string,
  params?: Record<string, unknown>,
  options: ApiCallOptions = {}
): Promise<T> {
  const { log = import.meta.env.DEV } = options;

  try {
    if (log) {
      logger.debug("API", `Calling ${command}`, params);
    }

    const result = await invoke<T>(command, params);

    if (log) {
      logger.debug("API", `${command} succeeded`);
    }

    return result;
  } catch (e) {
    logger.error("API", `${command} failed:`, e);
    throw new ApiError(command, e);
  }
}

/**
 * Type-safe wrapper for commands that return void.
 */
export async function apiCallVoid(
  command: string,
  params?: Record<string, unknown>,
  options: ApiCallOptions = {}
): Promise<void> {
  await apiCall<void>(command, params, options);
}

/**
 * Batch multiple API calls and return when all complete.
 * Individual failures don't stop other calls.
 *
 * @param calls - Array of [command, params] tuples.
 * @returns Array of results in the same order as calls.
 */
export async function apiBatch<T>(
  calls: Array<[string, Record<string, unknown>?]>
): Promise<PromiseSettledResult<T>[]> {
  return Promise.allSettled(
    calls.map(([command, params]) => apiCall<T>(command, params))
  );
}
