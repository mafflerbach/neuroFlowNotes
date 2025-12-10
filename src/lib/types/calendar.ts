/**
 * Calendar and scheduling types
 */

import type { NoteListItem } from "./note";

export interface ScheduleBlockDto {
  id: number;
  note_id: number | null;
  date: string;
  start_time: string;
  end_time: string;
  label: string | null;
  color: string | null;
  context: string | null;
  /** RFC 5545 recurrence rule (e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR") */
  rrule: string | null;
  /** True if this is an occurrence of a recurring block (not the master) */
  is_occurrence: boolean;
}

export interface CreateScheduleBlockRequest {
  note_id: number | null;
  date: string;
  start_time: string;
  end_time: string;
  label: string | null;
  color: string | null;
  context: string | null;
  /** RFC 5545 recurrence rule (e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR") */
  rrule: string | null;
}

export interface UpdateScheduleBlockRequest {
  id: number;
  note_id: number | null;
  date: string | null;
  start_time: string | null;
  end_time: string | null;
  label: string | null;
  color: string | null;
  context: string | null;
  /** RFC 5545 recurrence rule. Set to empty string to clear recurrence. */
  rrule: string | null;
}

export interface NoteForDate {
  note: NoteListItem;
  source: "scheduled" | "journal" | "created";
  schedule_block: ScheduleBlockDto | null;
}
