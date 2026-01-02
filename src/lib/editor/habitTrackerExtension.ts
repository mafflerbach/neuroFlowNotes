/**
 * Habit Tracker Embed Extension for CodeMirror
 * Renders ```habit-tracker``` code blocks with interactive habit tracking tables.
 *
 * Syntax example:
 * ```habit-tracker
 * habits:
 *   - "Exercise"
 *   - "Drink Water"
 * view: table
 * date_range: last_7_days
 * editable: true
 * show_summary: true
 * ```
 */

import {
  EditorView,
  ViewPlugin,
  Decoration,
  WidgetType,
} from "@codemirror/view";
import type { ViewUpdate, DecorationSet } from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";
import type { EditorState } from "@codemirror/state";
import { executeHabitTrackerEmbed, toggleHabit, logHabitEntry } from "../services/api";
import type { HabitTrackerResponse, HabitWithEntries, HabitDto, HabitEntryDto } from "../types";
import { EditorCache } from "./cache";

// Pattern to match habit-tracker code block start
const HABIT_BLOCK_START = /^```habit-tracker\s*$/;
const HABIT_BLOCK_END = /^```\s*$/;

interface HabitBlock {
  startLine: number;
  endLine: number;
  from: number;
  to: number;
  yamlContent: string;
}

/**
 * Find habit-tracker blocks in the document
 */
function findHabitBlocks(state: EditorState): HabitBlock[] {
  const blocks: HabitBlock[] = [];
  const doc = state.doc;
  let inHabitBlock = false;
  let blockStartLine = 0;
  let blockStartFrom = 0;
  let yamlLines: string[] = [];

  for (let i = 1; i <= doc.lines; i++) {
    const line = doc.line(i);
    const text = line.text;

    if (!inHabitBlock && HABIT_BLOCK_START.test(text)) {
      inHabitBlock = true;
      blockStartLine = i;
      blockStartFrom = line.from;
      yamlLines = [];
    } else if (inHabitBlock && HABIT_BLOCK_END.test(text)) {
      blocks.push({
        startLine: blockStartLine,
        endLine: i,
        from: blockStartFrom,
        to: line.to,
        yamlContent: yamlLines.join("\n"),
      });
      inHabitBlock = false;
    } else if (inHabitBlock) {
      yamlLines.push(text);
    }
  }

  return blocks;
}

/**
 * Cache for habit tracker results
 */
const habitResultCache = new EditorCache<HabitTrackerResponse>(5000); // 5 seconds TTL

async function getHabitResults(yamlContent: string): Promise<HabitTrackerResponse> {
  const cached = habitResultCache.get(yamlContent);
  if (cached) {
    return cached;
  }

  try {
    const response = await executeHabitTrackerEmbed(yamlContent);
    habitResultCache.set(yamlContent, response);
    return response;
  } catch (e) {
    return {
      query: {
        habits: [],
        view: "table",
        orientation: "horizontal",
        date_range: "last7_days",
        editable: true,
        show_summary: true,
      },
      habits: [],
      date_range_start: "",
      date_range_end: "",
      error: `Failed to execute habit tracker: ${e}`,
    };
  }
}

/**
 * Invalidate the habit cache (call when entries are updated)
 */
export function invalidateHabitCache(): void {
  habitResultCache.clear();
}

/**
 * Widget for hidden lines
 */
class HiddenLineWidget extends WidgetType {
  eq(_other: HiddenLineWidget): boolean {
    return true;
  }

  toDOM(): HTMLElement {
    const span = document.createElement("span");
    span.className = "cm-habit-hidden-line";
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

/**
 * Widget for rendering habit tracker results
 */
class HabitTrackerWidget extends WidgetType {
  private response: HabitTrackerResponse | null = null;
  private loading = true;
  private element: HTMLElement | null = null;

  constructor(private block: HabitBlock) {
    super();
    this.loadResults();
  }

  private async loadResults() {
    this.loading = true;
    this.response = await getHabitResults(this.block.yamlContent);
    this.loading = false;
    if (this.element) {
      this.updateElement();
    }
  }

  eq(other: HabitTrackerWidget): boolean {
    return this.block.yamlContent === other.block.yamlContent;
  }

  toDOM(): HTMLElement {
    const wrapper = document.createElement("div");
    wrapper.className = "cm-habit-embed";
    this.element = wrapper;
    this.updateElement();
    return wrapper;
  }

  private updateElement() {
    if (!this.element) return;

    this.element.innerHTML = "";

    if (this.loading) {
      const loadingEl = document.createElement("div");
      loadingEl.className = "cm-habit-embed-loading";
      loadingEl.textContent = "Loading habits...";
      this.element.appendChild(loadingEl);
      return;
    }

    if (!this.response) {
      return;
    }

    if (this.response.error) {
      const errorEl = document.createElement("div");
      errorEl.className = "cm-habit-embed-error";
      errorEl.textContent = this.response.error;
      this.element.appendChild(errorEl);
      return;
    }

    // Header
    const header = document.createElement("div");
    header.className = "cm-habit-embed-header";

    const icon = document.createElement("span");
    icon.className = "cm-habit-embed-icon";
    icon.textContent = "✓";
    header.appendChild(icon);

    const title = document.createElement("span");
    title.className = "cm-habit-embed-title";
    title.textContent = `Habit Tracker (${this.response.date_range_start} - ${this.response.date_range_end})`;
    header.appendChild(title);

    this.element.appendChild(header);

    // Content based on view type
    if (this.response.habits.length === 0) {
      const emptyEl = document.createElement("div");
      emptyEl.className = "cm-habit-embed-empty";
      emptyEl.textContent = "No habits found";
      this.element.appendChild(emptyEl);
      return;
    }

    const viewType = this.response.query.view;
    if (viewType === "table") {
      const orientation = this.response.query.orientation || "horizontal";
      if (orientation === "vertical") {
        this.renderTableVertical();
      } else {
        this.renderTable();
      }
    } else if (viewType === "list") {
      this.renderList();
    } else if (viewType === "streak") {
      this.renderStreak();
    } else if (viewType === "calendar") {
      this.renderCalendar();
    } else {
      this.renderTable(); // Default
    }

    // Summary row
    if (this.response.query.show_summary) {
      this.renderSummary();
    }
  }

  private getDatesInRange(): string[] {
    if (!this.response) return [];

    const dates: string[] = [];
    const start = new Date(this.response.date_range_start);
    const end = new Date(this.response.date_range_end);

    for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
      dates.push(d.toISOString().split("T")[0]);
    }

    return dates;
  }

  private formatDateHeader(dateStr: string): string {
    const date = new Date(dateStr);
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const diffDays = Math.floor((date.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === -1) return "Yesterday";
    if (diffDays === 1) return "Tomorrow";

    // Return short format: Mon, Tue, etc.
    return date.toLocaleDateString("en-US", { weekday: "short" });
  }

  private formatDateSubheader(dateStr: string): string {
    const date = new Date(dateStr);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  }

  private getEntriesForDate(habitWithEntries: HabitWithEntries, date: string): HabitEntryDto[] {
    const entry = habitWithEntries.entries_by_date.find(([d]) => d === date);
    return entry ? entry[1] : [];
  }

  private renderTable() {
    if (!this.element || !this.response) return;

    const dates = this.getDatesInRange();
    const table = document.createElement("table");
    table.className = "cm-habit-table";

    // Header row
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");

    // Habit name column
    const thName = document.createElement("th");
    thName.textContent = "Habit";
    thName.className = "cm-habit-name-header";
    headerRow.appendChild(thName);

    // Date columns
    for (const date of dates) {
      const th = document.createElement("th");
      th.className = "cm-habit-date-header";

      const dayName = document.createElement("div");
      dayName.className = "cm-habit-day-name";
      dayName.textContent = this.formatDateHeader(date);
      th.appendChild(dayName);

      const dayNum = document.createElement("div");
      dayNum.className = "cm-habit-day-num";
      dayNum.textContent = this.formatDateSubheader(date);
      th.appendChild(dayNum);

      headerRow.appendChild(th);
    }

    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Body rows
    const tbody = document.createElement("tbody");

    for (const habitWithEntries of this.response.habits) {
      const row = document.createElement("tr");

      // Habit name cell
      const tdName = document.createElement("td");
      tdName.className = "cm-habit-name-cell";

      if (habitWithEntries.habit.color) {
        const colorDot = document.createElement("span");
        colorDot.className = "cm-habit-color-dot";
        colorDot.style.backgroundColor = habitWithEntries.habit.color;
        tdName.appendChild(colorDot);
      }

      const nameSpan = document.createElement("span");
      nameSpan.textContent = habitWithEntries.habit.name;
      tdName.appendChild(nameSpan);

      row.appendChild(tdName);

      // Date cells
      for (const date of dates) {
        const td = document.createElement("td");
        td.className = "cm-habit-cell";
        this.renderCell(td, habitWithEntries.habit, habitWithEntries, date);
        row.appendChild(td);
      }

      tbody.appendChild(row);
    }

    table.appendChild(tbody);
    this.element.appendChild(table);
  }

  private renderTableVertical() {
    if (!this.element || !this.response) return;

    const dates = this.getDatesInRange();
    const habits = this.response.habits;
    const table = document.createElement("table");
    table.className = "cm-habit-table cm-habit-table-vertical";

    // Header row with habit names
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");

    // Date column header
    const thDate = document.createElement("th");
    thDate.textContent = "Date";
    thDate.className = "cm-habit-date-header-vertical";
    headerRow.appendChild(thDate);

    // Habit name columns
    for (const habitWithEntries of habits) {
      const th = document.createElement("th");
      th.className = "cm-habit-name-header-vertical";

      if (habitWithEntries.habit.color) {
        const colorDot = document.createElement("span");
        colorDot.className = "cm-habit-color-dot";
        colorDot.style.backgroundColor = habitWithEntries.habit.color;
        th.appendChild(colorDot);
      }

      const nameSpan = document.createElement("span");
      nameSpan.textContent = habitWithEntries.habit.name;
      th.appendChild(nameSpan);

      headerRow.appendChild(th);
    }

    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Body rows (one per date)
    const tbody = document.createElement("tbody");

    for (const date of dates) {
      const row = document.createElement("tr");

      // Date cell
      const tdDate = document.createElement("td");
      tdDate.className = "cm-habit-date-cell-vertical";

      const dayName = document.createElement("div");
      dayName.className = "cm-habit-day-name";
      dayName.textContent = this.formatDateHeader(date);
      tdDate.appendChild(dayName);

      const dayNum = document.createElement("div");
      dayNum.className = "cm-habit-day-num";
      dayNum.textContent = this.formatDateSubheader(date);
      tdDate.appendChild(dayNum);

      row.appendChild(tdDate);

      // Habit cells
      for (const habitWithEntries of habits) {
        const td = document.createElement("td");
        td.className = "cm-habit-cell";
        this.renderCell(td, habitWithEntries.habit, habitWithEntries, date);
        row.appendChild(td);
      }

      tbody.appendChild(row);
    }

    table.appendChild(tbody);
    this.element.appendChild(table);
  }

  private renderCell(
    td: HTMLElement,
    habit: HabitDto,
    habitWithEntries: HabitWithEntries,
    date: string
  ) {
    const entries = this.getEntriesForDate(habitWithEntries, date);
    const isEditable = this.response?.query.editable ?? true;

    if (habit.habit_type === "boolean") {
      const hasEntry = entries.length > 0 && entries[0].value === "true";
      const checkbox = document.createElement("button");
      checkbox.className = `cm-habit-checkbox ${hasEntry ? "checked" : ""}`;
      checkbox.innerHTML = hasEntry ? "✓" : "";

      if (isEditable) {
        checkbox.onclick = async (e) => {
          e.preventDefault();
          e.stopPropagation();

          try {
            const nowChecked = await toggleHabit(habit.id, date);
            checkbox.className = `cm-habit-checkbox ${nowChecked ? "checked" : ""}`;
            checkbox.innerHTML = nowChecked ? "✓" : "";
            // Invalidate cache after toggle
            invalidateHabitCache();
          } catch (err) {
            console.error("Failed to toggle habit:", err);
          }
        };
      } else {
        checkbox.disabled = true;
      }

      td.appendChild(checkbox);
    } else if (habit.habit_type === "number") {
      const value = entries.length > 0 ? entries[0].value : "";

      if (isEditable) {
        const input = document.createElement("input");
        input.type = "number";
        input.className = "cm-habit-number-input";
        input.value = value;
        input.placeholder = habit.target_value?.toString() || "0";

        input.onblur = async () => {
          const newValue = input.value;
          if (newValue !== value) {
            try {
              await logHabitEntry({
                habit_id: habit.id,
                date,
                value: newValue || "0",
              });
              invalidateHabitCache();
            } catch (err) {
              console.error("Failed to log habit entry:", err);
            }
          }
        };

        input.onkeydown = (e) => {
          if (e.key === "Enter") {
            input.blur();
          }
        };

        td.appendChild(input);

        // Show target indicator
        if (habit.target_value && parseFloat(value || "0") >= habit.target_value) {
          td.classList.add("cm-habit-target-reached");
        }
      } else {
        td.textContent = value || "-";
      }
    } else if (habit.habit_type === "text") {
      const value = entries.length > 0 ? entries[0].value : "";

      if (isEditable) {
        const input = document.createElement("input");
        input.type = "text";
        input.className = "cm-habit-text-input";
        input.value = value;

        input.onblur = async () => {
          const newValue = input.value;
          if (newValue !== value) {
            try {
              await logHabitEntry({
                habit_id: habit.id,
                date,
                value: newValue,
              });
              invalidateHabitCache();
            } catch (err) {
              console.error("Failed to log habit entry:", err);
            }
          }
        };

        input.onkeydown = (e) => {
          if (e.key === "Enter") {
            input.blur();
          }
        };

        td.appendChild(input);
      } else {
        td.textContent = value || "-";
      }
    } else if (habit.habit_type === "rating") {
      const value = entries.length > 0 ? parseInt(entries[0].value) : 0;
      const ratingContainer = document.createElement("div");
      ratingContainer.className = "cm-habit-rating";

      for (let i = 1; i <= 5; i++) {
        const star = document.createElement("span");
        star.className = `cm-habit-star ${i <= value ? "filled" : ""}`;
        star.textContent = i <= value ? "★" : "☆";

        if (isEditable) {
          star.onclick = async (e) => {
            e.preventDefault();
            e.stopPropagation();

            try {
              await logHabitEntry({
                habit_id: habit.id,
                date,
                value: i.toString(),
              });
              invalidateHabitCache();

              // Update stars visually
              const stars = ratingContainer.querySelectorAll(".cm-habit-star");
              stars.forEach((s, idx) => {
                s.className = `cm-habit-star ${idx < i ? "filled" : ""}`;
                s.textContent = idx < i ? "★" : "☆";
              });
            } catch (err) {
              console.error("Failed to log rating:", err);
            }
          };
        }

        ratingContainer.appendChild(star);
      }

      td.appendChild(ratingContainer);
    }
  }

  private renderList() {
    if (!this.element || !this.response) return;

    const list = document.createElement("ul");
    list.className = "cm-habit-list";

    const today = new Date().toISOString().split("T")[0];

    for (const habitWithEntries of this.response.habits) {
      const li = document.createElement("li");
      li.className = "cm-habit-list-item";

      // Color dot
      if (habitWithEntries.habit.color) {
        const colorDot = document.createElement("span");
        colorDot.className = "cm-habit-color-dot";
        colorDot.style.backgroundColor = habitWithEntries.habit.color;
        li.appendChild(colorDot);
      }

      // Habit name
      const nameSpan = document.createElement("span");
      nameSpan.className = "cm-habit-list-name";
      nameSpan.textContent = habitWithEntries.habit.name;
      li.appendChild(nameSpan);

      // Today's status
      const entries = this.getEntriesForDate(habitWithEntries, today);
      const statusSpan = document.createElement("span");
      statusSpan.className = "cm-habit-list-status";

      if (habitWithEntries.habit.habit_type === "boolean") {
        const hasEntry = entries.length > 0 && entries[0].value === "true";
        statusSpan.textContent = hasEntry ? "Done" : "Not done";
        statusSpan.classList.add(hasEntry ? "done" : "pending");
      } else if (entries.length > 0) {
        statusSpan.textContent = entries[0].value;
      } else {
        statusSpan.textContent = "No entry";
        statusSpan.classList.add("pending");
      }

      li.appendChild(statusSpan);
      list.appendChild(li);
    }

    this.element.appendChild(list);
  }

  private renderStreak() {
    if (!this.element || !this.response) return;

    const streakContainer = document.createElement("div");
    streakContainer.className = "cm-habit-streak-container";

    for (const habitWithEntries of this.response.habits) {
      const streakItem = document.createElement("div");
      streakItem.className = "cm-habit-streak-item";

      // Habit name
      const nameDiv = document.createElement("div");
      nameDiv.className = "cm-habit-streak-name";

      if (habitWithEntries.habit.color) {
        const colorDot = document.createElement("span");
        colorDot.className = "cm-habit-color-dot";
        colorDot.style.backgroundColor = habitWithEntries.habit.color;
        nameDiv.appendChild(colorDot);
      }

      const nameSpan = document.createElement("span");
      nameSpan.textContent = habitWithEntries.habit.name;
      nameDiv.appendChild(nameSpan);
      streakItem.appendChild(nameDiv);

      // Calculate streak
      const streak = this.calculateStreak(habitWithEntries);
      const streakDisplay = document.createElement("div");
      streakDisplay.className = "cm-habit-streak-count";

      const streakNumber = document.createElement("span");
      streakNumber.className = "cm-habit-streak-number";
      streakNumber.textContent = streak.toString();
      streakDisplay.appendChild(streakNumber);

      const streakLabel = document.createElement("span");
      streakLabel.className = "cm-habit-streak-label";
      streakLabel.textContent = streak === 1 ? " day" : " days";
      streakDisplay.appendChild(streakLabel);

      streakItem.appendChild(streakDisplay);
      streakContainer.appendChild(streakItem);
    }

    this.element.appendChild(streakContainer);
  }

  private calculateStreak(habitWithEntries: HabitWithEntries): number {
    if (habitWithEntries.habit.habit_type !== "boolean") {
      return 0; // Streak only makes sense for boolean habits
    }

    let streak = 0;
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    // Start from today and go backwards
    for (let i = 0; i < 365; i++) {
      const checkDate = new Date(today);
      checkDate.setDate(checkDate.getDate() - i);
      const dateStr = checkDate.toISOString().split("T")[0];

      const entries = this.getEntriesForDate(habitWithEntries, dateStr);
      const hasEntry = entries.length > 0 && entries[0].value === "true";

      if (hasEntry) {
        streak++;
      } else if (i > 0) {
        // If we skipped today, continue checking
        // But if we miss any other day, break
        break;
      }
    }

    return streak;
  }

  private renderCalendar() {
    if (!this.element || !this.response) return;

    const calendarContainer = document.createElement("div");
    calendarContainer.className = "cm-habit-calendar-container";

    for (const habitWithEntries of this.response.habits) {
      const habitSection = document.createElement("div");
      habitSection.className = "cm-habit-calendar-section";

      // Habit name
      const nameDiv = document.createElement("div");
      nameDiv.className = "cm-habit-calendar-name";

      if (habitWithEntries.habit.color) {
        const colorDot = document.createElement("span");
        colorDot.className = "cm-habit-color-dot";
        colorDot.style.backgroundColor = habitWithEntries.habit.color;
        nameDiv.appendChild(colorDot);
      }

      const nameSpan = document.createElement("span");
      nameSpan.textContent = habitWithEntries.habit.name;
      nameDiv.appendChild(nameSpan);
      habitSection.appendChild(nameDiv);

      // Calendar grid (simplified heatmap)
      const grid = document.createElement("div");
      grid.className = "cm-habit-calendar-grid";

      const dates = this.getDatesInRange();
      for (const date of dates) {
        const cell = document.createElement("div");
        cell.className = "cm-habit-calendar-cell";
        cell.title = date;

        const entries = this.getEntriesForDate(habitWithEntries, date);

        if (habitWithEntries.habit.habit_type === "boolean") {
          const hasEntry = entries.length > 0 && entries[0].value === "true";
          if (hasEntry) {
            cell.classList.add("completed");
            cell.style.backgroundColor = habitWithEntries.habit.color || "var(--green)";
          }
        } else if (entries.length > 0) {
          cell.classList.add("has-value");
          cell.style.backgroundColor = habitWithEntries.habit.color || "var(--blue)";
          cell.style.opacity = "0.7";
        }

        grid.appendChild(cell);
      }

      habitSection.appendChild(grid);
      calendarContainer.appendChild(habitSection);
    }

    this.element.appendChild(calendarContainer);
  }

  private renderSummary() {
    if (!this.element || !this.response) return;

    const summary = document.createElement("div");
    summary.className = "cm-habit-summary";

    const dates = this.getDatesInRange();
    const totalDays = dates.length;

    for (const habitWithEntries of this.response.habits) {
      if (habitWithEntries.habit.habit_type !== "boolean") continue;

      let completedDays = 0;
      for (const date of dates) {
        const entries = this.getEntriesForDate(habitWithEntries, date);
        if (entries.length > 0 && entries[0].value === "true") {
          completedDays++;
        }
      }

      const percentage = Math.round((completedDays / totalDays) * 100);

      const summaryItem = document.createElement("div");
      summaryItem.className = "cm-habit-summary-item";

      const name = document.createElement("span");
      name.className = "cm-habit-summary-name";
      name.textContent = habitWithEntries.habit.name;
      summaryItem.appendChild(name);

      const stats = document.createElement("span");
      stats.className = "cm-habit-summary-stats";
      stats.textContent = `${completedDays}/${totalDays} (${percentage}%)`;
      summaryItem.appendChild(stats);

      summary.appendChild(summaryItem);
    }

    if (summary.children.length > 0) {
      this.element.appendChild(summary);
    }
  }

  ignoreEvent(): boolean {
    return true;
  }

  destroy() {
    this.element = null;
  }
}

/**
 * Get active lines (where cursor is)
 */
function getActiveLines(state: EditorState): Set<number> {
  const activeLines = new Set<number>();
  for (const range of state.selection.ranges) {
    const startLine = state.doc.lineAt(range.from).number;
    const endLine = state.doc.lineAt(range.to).number;
    for (let line = startLine; line <= endLine; line++) {
      activeLines.add(line);
    }
  }
  return activeLines;
}

interface DecorationEntry {
  from: number;
  to: number;
  decoration: Decoration;
  isLine: boolean;
}

/**
 * Create decorations for habit blocks
 */
function createDecorations(view: EditorView): DecorationSet {
  const blocks = findHabitBlocks(view.state);
  const activeLines = getActiveLines(view.state);
  const allDecorations: DecorationEntry[] = [];

  for (const block of blocks) {
    // Check if cursor is in this block
    let cursorInBlock = false;
    for (let line = block.startLine; line <= block.endLine; line++) {
      if (activeLines.has(line)) {
        cursorInBlock = true;
        break;
      }
    }

    // If cursor is in block, show raw markdown
    if (cursorInBlock) {
      continue;
    }

    const doc = view.state.doc;
    const hiddenWidget = new HiddenLineWidget();

    // Replace the opening ```habit-tracker line
    const startLine = doc.line(block.startLine);
    allDecorations.push({
      from: startLine.from,
      to: startLine.to,
      decoration: Decoration.replace({ widget: hiddenWidget }),
      isLine: false,
    });

    // Hide all the intermediate YAML lines
    for (let lineNum = block.startLine + 1; lineNum < block.endLine; lineNum++) {
      const line = doc.line(lineNum);
      allDecorations.push({
        from: line.from,
        to: line.to,
        decoration: Decoration.replace({ widget: new HiddenLineWidget() }),
        isLine: false,
      });
    }

    // Replace the closing ``` with the widget
    const endLine = doc.line(block.endLine);
    const widget = new HabitTrackerWidget(block);
    allDecorations.push({
      from: endLine.from,
      to: endLine.to,
      decoration: Decoration.replace({ widget }),
      isLine: false,
    });
  }

  // Sort decorations by position
  allDecorations.sort((a, b) => {
    if (a.from !== b.from) return a.from - b.from;
    if (a.isLine !== b.isLine) return a.isLine ? -1 : 1;
    return a.to - b.to;
  });

  // Build the decoration set
  const builder = new RangeSetBuilder<Decoration>();
  for (const deco of allDecorations) {
    builder.add(deco.from, deco.to, deco.decoration);
  }

  return builder.finish();
}

/**
 * ViewPlugin that manages habit tracker embed decorations
 */
const habitTrackerPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = createDecorations(view);
    }

    update(update: ViewUpdate) {
      if (
        update.docChanged ||
        update.selectionSet ||
        update.viewportChanged
      ) {
        this.decorations = createDecorations(update.view);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);

// Inject habit tracker styles
const injectStyles = () => {
  if (typeof document === "undefined") return;
  if (document.getElementById("habit-tracker-extension-styles")) return;

  const style = document.createElement("style");
  style.id = "habit-tracker-extension-styles";
  style.textContent = `
    /* Hide the empty .cm-line elements for replaced lines */
    .cm-line:has(.cm-habit-hidden-line) {
      display: none !important;
      height: 0 !important;
      min-height: 0 !important;
      padding: 0 !important;
      margin: 0 !important;
      line-height: 0 !important;
    }

    .cm-habit-hidden-line {
      display: none;
    }

    .cm-habit-embed {
      margin: 8px 0;
      border: 1px solid var(--border-default);
      border-radius: var(--radius-md);
      background: var(--bg-surface);
      overflow: hidden;
    }

    .cm-habit-embed-header {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      padding: var(--spacing-2) var(--spacing-3);
      background: var(--bg-surface-sunken);
      border-bottom: 1px solid var(--border-default);
    }

    .cm-habit-embed-icon {
      font-size: 1.1em;
      color: var(--green);
    }

    .cm-habit-embed-title {
      font-weight: var(--font-weight-medium);
      font-size: var(--font-size-sm);
      color: var(--text-muted);
    }

    .cm-habit-embed-loading,
    .cm-habit-embed-empty {
      padding: var(--spacing-4);
      text-align: center;
      color: var(--text-muted);
      font-style: italic;
    }

    .cm-habit-embed-error {
      padding: var(--spacing-3);
      color: var(--color-error);
      background: var(--color-error-light);
      border-radius: var(--radius-sm);
      margin: var(--spacing-2);
    }

    /* Table styles */
    .cm-habit-table {
      width: 100%;
      border-collapse: collapse;
      font-size: var(--font-size-sm);
    }

    .cm-habit-table th {
      padding: var(--spacing-2);
      background: var(--bg-surface-sunken);
      border-bottom: 1px solid var(--border-default);
      font-weight: var(--font-weight-medium);
      color: var(--text-muted);
    }

    .cm-habit-name-header {
      text-align: left;
      min-width: 120px;
    }

    .cm-habit-date-header {
      text-align: center;
      min-width: 60px;
    }

    .cm-habit-day-name {
      font-size: var(--font-size-xs);
      color: var(--text-primary);
    }

    .cm-habit-day-num {
      font-size: var(--font-size-xs);
      color: var(--text-muted);
    }

    /* Vertical table styles */
    .cm-habit-table-vertical .cm-habit-date-header-vertical {
      text-align: left;
      min-width: 80px;
    }

    .cm-habit-table-vertical .cm-habit-name-header-vertical {
      text-align: center;
      min-width: 60px;
    }

    .cm-habit-table-vertical .cm-habit-date-cell-vertical {
      text-align: left;
      padding: var(--spacing-2);
    }

    .cm-habit-table td {
      padding: var(--spacing-2);
      border-bottom: 1px solid var(--border-subtle);
    }

    .cm-habit-name-cell {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
    }

    .cm-habit-color-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      flex-shrink: 0;
    }

    .cm-habit-cell {
      text-align: center;
    }

    .cm-habit-cell.cm-habit-target-reached {
      background: var(--green-light, rgba(0, 255, 0, 0.1));
    }

    /* Checkbox */
    .cm-habit-checkbox {
      width: 24px;
      height: 24px;
      border: 2px solid var(--border-medium);
      border-radius: var(--radius-sm);
      background: transparent;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      font-size: 14px;
      color: white;
      transition: all 0.15s;
    }

    .cm-habit-checkbox:hover:not(:disabled) {
      border-color: var(--color-primary);
    }

    .cm-habit-checkbox.checked {
      background: var(--green);
      border-color: var(--green);
    }

    .cm-habit-checkbox:disabled {
      cursor: default;
      opacity: 0.6;
    }

    /* Number input */
    .cm-habit-number-input,
    .cm-habit-text-input {
      width: 50px;
      padding: var(--spacing-1);
      border: 1px solid var(--border-light);
      border-radius: var(--radius-sm);
      background: var(--input-bg);
      color: var(--input-text);
      text-align: center;
      font-size: var(--font-size-sm);
    }

    .cm-habit-number-input:focus,
    .cm-habit-text-input:focus {
      outline: none;
      border-color: var(--color-primary);
    }

    .cm-habit-text-input {
      width: 80px;
    }

    /* Rating */
    .cm-habit-rating {
      display: inline-flex;
      gap: 2px;
    }

    .cm-habit-star {
      cursor: pointer;
      font-size: 14px;
      color: var(--text-muted);
      transition: color 0.15s;
    }

    .cm-habit-star.filled {
      color: var(--yellow);
    }

    .cm-habit-star:hover {
      color: var(--yellow);
    }

    /* List view */
    .cm-habit-list {
      list-style: none;
      margin: 0;
      padding: 0;
    }

    .cm-habit-list-item {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      padding: var(--spacing-2) var(--spacing-3);
      border-bottom: 1px solid var(--border-subtle);
    }

    .cm-habit-list-item:last-child {
      border-bottom: none;
    }

    .cm-habit-list-name {
      flex: 1;
    }

    .cm-habit-list-status {
      font-size: var(--font-size-sm);
      padding: 2px 8px;
      border-radius: var(--radius-sm);
      background: var(--bg-tertiary);
    }

    .cm-habit-list-status.done {
      background: var(--green);
      color: var(--base);
    }

    .cm-habit-list-status.pending {
      color: var(--text-muted);
    }

    /* Streak view */
    .cm-habit-streak-container {
      padding: var(--spacing-3);
      display: flex;
      flex-direction: column;
      gap: var(--spacing-3);
    }

    .cm-habit-streak-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: var(--spacing-2);
      background: var(--bg-surface-sunken);
      border-radius: var(--radius-sm);
    }

    .cm-habit-streak-name {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
    }

    .cm-habit-streak-count {
      display: flex;
      align-items: baseline;
      gap: 2px;
    }

    .cm-habit-streak-number {
      font-size: var(--font-size-xl);
      font-weight: var(--font-weight-bold);
      color: var(--text-primary);
    }

    .cm-habit-streak-label {
      font-size: var(--font-size-sm);
      color: var(--text-muted);
    }

    /* Calendar/heatmap view */
    .cm-habit-calendar-container {
      padding: var(--spacing-3);
      display: flex;
      flex-direction: column;
      gap: var(--spacing-3);
    }

    .cm-habit-calendar-section {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-2);
    }

    .cm-habit-calendar-name {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      font-size: var(--font-size-sm);
      font-weight: var(--font-weight-medium);
    }

    .cm-habit-calendar-grid {
      display: flex;
      gap: 3px;
      flex-wrap: wrap;
    }

    .cm-habit-calendar-cell {
      width: 14px;
      height: 14px;
      border-radius: 2px;
      background: var(--bg-surface-sunken);
      border: 1px solid var(--border-subtle);
    }

    .cm-habit-calendar-cell.completed {
      border-color: transparent;
    }

    .cm-habit-calendar-cell.has-value {
      border-color: transparent;
    }

    /* Summary */
    .cm-habit-summary {
      padding: var(--spacing-2) var(--spacing-3);
      background: var(--bg-surface-sunken);
      border-top: 1px solid var(--border-default);
      display: flex;
      flex-wrap: wrap;
      gap: var(--spacing-3);
    }

    .cm-habit-summary-item {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      font-size: var(--font-size-sm);
    }

    .cm-habit-summary-name {
      color: var(--text-muted);
    }

    .cm-habit-summary-stats {
      font-weight: var(--font-weight-medium);
      color: var(--text-primary);
    }
  `;
  document.head.appendChild(style);
};

/**
 * Extension that provides habit tracker embed rendering.
 */
export function habitTrackerExtension() {
  injectStyles();
  return [habitTrackerPlugin];
}
