/**
 * Todos API - todo management operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { TodoDto, TaskQuery, TaskWithContext } from "../../types";

export async function getTodosForNote(noteId: number): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_todos_for_note", { noteId });
}

export async function toggleTodo(todoId: number, completed: boolean): Promise<void> {
  return invoke("toggle_todo", { todoId, completed });
}

export async function getIncompleteTodos(): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_incomplete_todos");
}

/**
 * Query tasks with filters, returning enriched context from parent notes.
 */
export async function queryTasks(query: TaskQuery = {}): Promise<TaskWithContext[]> {
  return invoke<TaskWithContext[]>("query_tasks", { query });
}

/**
 * Get all distinct contexts used in tasks.
 */
export async function getTaskContexts(): Promise<string[]> {
  return invoke<string[]>("get_task_contexts");
}
