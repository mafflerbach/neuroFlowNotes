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

/**
 * Extract the first H1 title from markdown content.
 * Returns null if no H1 found.
 */
export function extractH1Title(content: string): string | null {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].trim() : null;
}

/**
 * Generate a safe filename from a title (like Obsidian does).
 * Converts the title to a filename-safe string.
 */
export function titleToFilename(title: string): string {
  // Replace characters that are invalid in filenames
  // Keep: letters, numbers, spaces, dashes, underscores
  // Replace others with dashes
  const safe = title
    .replace(/[<>:"/\\|?*]/g, "-") // Invalid filename chars
    .replace(/\s+/g, " ") // Normalize whitespace
    .trim();
  return `${safe}.md`;
}

/**
 * Get the directory part of a path.
 */
export function getDirectory(path: string): string {
  const lastSlash = path.lastIndexOf("/");
  return lastSlash >= 0 ? path.substring(0, lastSlash) : "";
}

/**
 * Generate a new path for a note based on its title, keeping it in the same directory.
 */
export function generatePathFromTitle(currentPath: string, title: string): string {
  const dir = getDirectory(currentPath);
  const filename = titleToFilename(title);
  return dir ? `${dir}/${filename}` : filename;
}

/**
 * Replace the first H1 title in markdown content with a new title.
 * If no H1 exists, prepends one to the content.
 */
export function replaceH1Title(content: string, newTitle: string): string {
  const h1Regex = /^#\s+.+$/m;
  if (h1Regex.test(content)) {
    return content.replace(h1Regex, `# ${newTitle}`);
  }
  // No H1 found, prepend one
  return `# ${newTitle}\n\n${content}`;
}
