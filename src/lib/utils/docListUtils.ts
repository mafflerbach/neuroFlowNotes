/**
 * Utilities for document list processing.
 * Handles deduplication, source separation, and data transformation.
 */

import type { NoteListItem, ScheduleBlockDto, NoteForDate } from "../types";

export interface DocWithSource {
  note: NoteListItem;
  source: "scheduled" | "journal" | "created";
  scheduleBlock?: ScheduleBlockDto;
  date?: string;
}

export interface SeparatedDocs {
  scheduledDocs: DocWithSource[];
  journalDocs: DocWithSource[];
  createdDocs: DocWithSource[];
}

/**
 * Deduplicate an array of items by a key function.
 * Returns a new array with only the first occurrence of each key.
 */
export function deduplicateBy<T, K>(items: T[], keyFn: (item: T) => K): T[] {
  const seen = new Set<K>();
  return items.filter((item) => {
    const key = keyFn(item);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

/**
 * Deduplicate notes by their ID.
 */
export function deduplicateNotes<T extends { note: NoteListItem }>(items: T[]): T[] {
  return deduplicateBy(items, (item) => item.note.id);
}

/**
 * Separate notes by source and deduplicate within each category.
 * Returns three arrays: scheduled, journal, and created docs.
 */
export function separateAndDeduplicateNotes(notes: NoteForDate[]): SeparatedDocs {
  const scheduledDocs = deduplicateNotes(
    notes
      .filter((n): n is NoteForDate & { source: "scheduled" } => n.source === "scheduled")
      .map((n) => ({
        note: n.note,
        source: "scheduled" as const,
        scheduleBlock: n.schedule_block ?? undefined,
      }))
  );

  const journalDocs = deduplicateNotes(
    notes
      .filter((n) => n.source === "journal")
      .map((n) => ({
        note: n.note,
        source: "journal" as const,
      }))
  );

  const createdDocs = deduplicateNotes(
    notes
      .filter((n) => n.source === "created")
      .map((n) => ({
        note: n.note,
        source: "created" as const,
      }))
  );

  return { scheduledDocs, journalDocs, createdDocs };
}

/**
 * Flatten notes from a date-keyed map into a single array.
 * Useful for week/month views where we need all notes.
 */
export function flattenNotesFromDateMap(
  dateMap: Map<string, NoteForDate[]> | Array<[string, NoteForDate[]]>
): NoteForDate[] {
  const notes: NoteForDate[] = [];
  const entries = dateMap instanceof Map ? dateMap.entries() : dateMap;

  for (const [, dayNotes] of entries) {
    notes.push(...dayNotes);
  }

  return notes;
}

/**
 * Get deduplicated notes for month display.
 * Returns notes sorted by date descending, with duplicates removed.
 */
export function getMonthNotesForDisplay(
  notesForMonth: Map<string, NoteListItem[]>
): Array<{ date: string; note: NoteListItem }> {
  const seen = new Set<number>();
  const notes: Array<{ date: string; note: NoteListItem }> = [];

  notesForMonth.forEach((noteList, dateKey) => {
    noteList.forEach((note) => {
      if (!seen.has(note.id)) {
        seen.add(note.id);
        notes.push({ date: dateKey, note });
      }
    });
  });

  // Sort by date descending
  notes.sort((a, b) => b.date.localeCompare(a.date));

  return notes;
}

/**
 * Generate a filename for a new note based on date, time, and label.
 */
export function generateNoteFilename(date: string, time: string, label: string): string {
  const timeStr = time.replace(":", "");
  const safeName = label.replace(/[^a-zA-Z0-9-_]/g, "-").toLowerCase();
  return `${date}-${timeStr}-${safeName}.md`;
}

/**
 * Generate initial content for a new note (without frontmatter).
 * Metadata should be stored in the database, not in the file.
 */
export function generateNoteContent(title: string): string {
  return `# ${title}\n\n`;
}
