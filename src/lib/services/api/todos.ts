/**
 * Todos API - todo management operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { TodoDto } from "../../types";

export async function getTodosForNote(noteId: number): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_todos_for_note", { noteId });
}

export async function toggleTodo(todoId: number, completed: boolean): Promise<void> {
  return invoke("toggle_todo", { todoId, completed });
}

export async function getIncompleteTodos(): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_incomplete_todos");
}
