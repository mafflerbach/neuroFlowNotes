/**
 * Tauri event subscriptions.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  NotesUpdatedPayload,
  NotesDeletedPayload,
  IndexCompletePayload,
  ImportProgress,
} from "../types";

export type EventCallback<T> = (payload: T) => void;

export function onNotesUpdated(callback: EventCallback<NotesUpdatedPayload>): Promise<UnlistenFn> {
  return listen<NotesUpdatedPayload>("notes:updated", (event) => {
    callback(event.payload);
  });
}

export function onNotesDeleted(callback: EventCallback<NotesDeletedPayload>): Promise<UnlistenFn> {
  return listen<NotesDeletedPayload>("notes:deleted", (event) => {
    callback(event.payload);
  });
}

export function onIndexComplete(callback: EventCallback<IndexCompletePayload>): Promise<UnlistenFn> {
  return listen<IndexCompletePayload>("index:complete", (event) => {
    callback(event.payload);
  });
}

export function onImportProgress(callback: EventCallback<ImportProgress>): Promise<UnlistenFn> {
  return listen<ImportProgress>("import:progress", (event) => {
    callback(event.payload);
  });
}
