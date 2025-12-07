/**
 * Calendar API - schedule blocks and notes by date
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  ScheduleBlockDto,
  CreateScheduleBlockRequest,
  UpdateScheduleBlockRequest,
  NoteForDate,
} from "../../types";

// Schedule Blocks

export async function createScheduleBlock(request: CreateScheduleBlockRequest): Promise<number> {
  return invoke<number>("create_schedule_block", { request });
}

export async function getScheduleBlocks(startDate: string, endDate: string): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks", { startDate, endDate });
}

export async function getScheduleBlocksForDate(date: string): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks_for_date", { date });
}

export async function updateScheduleBlock(request: UpdateScheduleBlockRequest): Promise<void> {
  return invoke("update_schedule_block", { request });
}

export async function getScheduleBlocksForNote(noteId: number): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks_for_note", { noteId });
}

export async function deleteScheduleBlock(id: number): Promise<void> {
  return invoke("delete_schedule_block", { id });
}

// Notes by Date

export async function getNotesForDate(date: string): Promise<NoteForDate[]> {
  return invoke<NoteForDate[]>("get_notes_for_date", { date });
}

export async function getNotesForDateRange(
  startDate: string,
  endDate: string
): Promise<[string, NoteForDate[]][]> {
  return invoke<[string, NoteForDate[]][]>("get_notes_for_date_range", { startDate, endDate });
}
