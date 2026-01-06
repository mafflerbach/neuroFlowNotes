/**
 * Query Embed Extension for CodeMirror
 * Renders ```query``` code blocks with live query results.
 *
 * Syntax example:
 * ```query
 * filters:
 *   - key: project
 *     operator: Equals
 *     value: "MyProject"
 * match_mode: All
 * result_type: Tasks
 * include_completed: false
 * limit: 20
 * view:
 *   view_type: Table
 *   columns:
 *     - description
 *     - priority
 *     - due_date
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
import { convertFileSrc } from "@tauri-apps/api/core";
import { executeQueryEmbed, setProperty } from "../services/api";
import type { QueryEmbedResponse, QueryResultItem, QueryViewConfig, KanbanConfig, InteractiveFilter, StatsConfig, CardConfig } from "../types";
import { workspaceStore } from "../stores/workspace.svelte";
import { vaultStore } from "../stores/vault.svelte";
import { EditorCache } from "./cache";

// Pattern to match query code block start
const QUERY_BLOCK_START = /^```query\s*$/;
const QUERY_BLOCK_END = /^```\s*$/;

interface QueryBlock {
  startLine: number;
  endLine: number;
  from: number;
  to: number;
  yamlContent: string;
}

/**
 * Find query blocks in the document
 */
function findQueryBlocks(state: EditorState): QueryBlock[] {
  const blocks: QueryBlock[] = [];
  const doc = state.doc;
  let inQueryBlock = false;
  let blockStartLine = 0;
  let blockStartFrom = 0;
  let yamlLines: string[] = [];

  for (let i = 1; i <= doc.lines; i++) {
    const line = doc.line(i);
    const text = line.text;

    if (!inQueryBlock && QUERY_BLOCK_START.test(text)) {
      inQueryBlock = true;
      blockStartLine = i;
      blockStartFrom = line.from;
      yamlLines = [];
    } else if (inQueryBlock && QUERY_BLOCK_END.test(text)) {
      blocks.push({
        startLine: blockStartLine,
        endLine: i,
        from: blockStartFrom,
        to: line.to,
        yamlContent: yamlLines.join("\n"),
      });
      inQueryBlock = false;
    } else if (inQueryBlock) {
      yamlLines.push(text);
    }
  }

  return blocks;
}

/**
 * Cache for query results to avoid re-fetching on every keystroke
 */
const queryResultCache = new EditorCache<QueryEmbedResponse>(5000); // 5 seconds TTL

async function getQueryResults(yamlContent: string): Promise<QueryEmbedResponse> {
  const cached = queryResultCache.get(yamlContent);
  if (cached) {
    return cached;
  }

  try {
    const response = await executeQueryEmbed(yamlContent);
    queryResultCache.set(yamlContent, response);
    return response;
  } catch (e) {
    return {
      query: {
        filters: [],
        match_mode: "All",
        result_type: "Tasks",
        include_completed: false,
        limit: 50,
        view: { view_type: "Table", columns: [], sort: null, kanban: null, card: null },
        tabs: [],
      },
      results: [],
      total_count: 0,
      tab_results: [],
      error: `Failed to execute query: ${e}`,
    };
  }
}

/**
 * Invalidate the query cache (call when notes are updated)
 */
export function invalidateQueryCache(): void {
  queryResultCache.clear();
}

/**
 * Widget for hidden lines (adds a marker class to collapse the line)
 */
class HiddenLineWidget extends WidgetType {
  eq(_other: HiddenLineWidget): boolean {
    return true;
  }

  toDOM(): HTMLElement {
    const span = document.createElement("span");
    span.className = "cm-query-hidden-line";
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

/**
 * Widget for rendering query results
 */
class QueryResultWidget extends WidgetType {
  private response: QueryEmbedResponse | null = null;
  private loading = true;
  private element: HTMLElement | null = null;
  private activeTabIndex = 0;
  // Interactive filter state per tab (tab index -> filter key -> selected values)
  private tabFilterState = new Map<number, Map<string, Set<string>>>();

  constructor(private block: QueryBlock) {
    super();
    this.loadResults();
  }

  private async loadResults() {
    this.loading = true;
    this.response = await getQueryResults(this.block.yamlContent);
    this.loading = false;
    if (this.element) {
      this.updateElement();
    }
  }

  eq(other: QueryResultWidget): boolean {
    return this.block.yamlContent === other.block.yamlContent;
  }

  toDOM(): HTMLElement {
    const wrapper = document.createElement("div");
    wrapper.className = "cm-query-embed";
    this.element = wrapper;
    this.updateElement();
    return wrapper;
  }

  private updateElement() {
    if (!this.element) return;

    this.element.innerHTML = "";

    if (this.loading) {
      const loadingEl = document.createElement("div");
      loadingEl.className = "cm-query-embed-loading";
      loadingEl.textContent = "Loading query results...";
      this.element.appendChild(loadingEl);
      return;
    }

    if (!this.response) {
      return;
    }

    if (this.response.error) {
      const errorEl = document.createElement("div");
      errorEl.className = "cm-query-embed-error";
      errorEl.textContent = this.response.error;
      this.element.appendChild(errorEl);
      return;
    }

    // Check if we have tabs
    const hasTabs = this.response.tab_results && this.response.tab_results.length > 0;

    if (hasTabs) {
      this.renderTabbedView();
    } else {
      this.renderSingleView();
    }
  }

  private renderSingleView() {
    if (!this.element || !this.response) return;

    // Header with count
    const header = document.createElement("div");
    header.className = "cm-query-embed-header";

    const icon = document.createElement("span");
    icon.className = "cm-query-embed-icon";
    icon.textContent = "📊";
    header.appendChild(icon);

    const title = document.createElement("span");
    title.className = "cm-query-embed-title";
    title.textContent = `Query Results (${this.response.total_count})`;
    header.appendChild(title);

    this.element.appendChild(header);

    // Results container
    const contentContainer = document.createElement("div");
    contentContainer.className = "cm-query-content";

    // Apply interactive filters to results
    const results = this.response.results;
    const filteredResults = this.applyInteractiveFilters(results);

    // Render stats bar if configured (using filtered results)
    if (this.response.query.view.stats?.show) {
      this.renderStatsBar(contentContainer, filteredResults, this.response.query.view.stats);
    }

    // Render interactive filters if configured
    if (this.response.query.view.interactive_filters && this.response.query.view.interactive_filters.length > 0) {
      this.renderInteractiveFilters(contentContainer, this.response.query.view.interactive_filters, results);
    }

    if (filteredResults.length === 0) {
      const emptyEl = document.createElement("div");
      emptyEl.className = "cm-query-embed-empty";
      emptyEl.textContent = results.length === 0 ? "No results found" : "No results match selected filters";
      contentContainer.appendChild(emptyEl);
    } else {
      const viewType = this.response.query.view.view_type;
      if (viewType === "Table") {
        this.renderTableInContainer(filteredResults, this.response.query.view, contentContainer);
      } else if (viewType === "Kanban") {
        this.renderKanbanInContainer(filteredResults, this.response.query.view, contentContainer);
      } else if (viewType === "Card") {
        this.renderCardInContainer(filteredResults, this.response.query.view, contentContainer);
      } else {
        this.renderListInContainer(filteredResults, contentContainer);
      }
    }

    this.element.appendChild(contentContainer);
  }

  private renderTabbedView() {
    if (!this.element || !this.response) return;

    const tabResults = this.response.tab_results;

    // Header with tabs
    const header = document.createElement("div");
    header.className = "cm-query-embed-header cm-query-embed-header-tabs";

    const icon = document.createElement("span");
    icon.className = "cm-query-embed-icon";
    icon.textContent = "📊";
    header.appendChild(icon);

    // Tab buttons container
    const tabsContainer = document.createElement("div");
    tabsContainer.className = "cm-query-tabs";

    tabResults.forEach((tabResult, index) => {
      const tabBtn = document.createElement("button");
      tabBtn.className = `cm-query-tab ${index === this.activeTabIndex ? "active" : ""}`;
      tabBtn.textContent = `${tabResult.name} (${tabResult.total_count})`;
      tabBtn.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.activeTabIndex = index;
        this.updateElement();
      };
      tabsContainer.appendChild(tabBtn);
    });

    header.appendChild(tabsContainer);
    this.element.appendChild(header);

    // Tab content
    const activeTab = tabResults[this.activeTabIndex];
    if (!activeTab) return;

    const contentContainer = document.createElement("div");
    contentContainer.className = "cm-query-tab-content";

    // Apply interactive filters to results
    const filteredResults = this.applyInteractiveFilters(activeTab.results);

    // Render stats bar if configured (before filters, using filtered results)
    if (activeTab.view.stats?.show) {
      this.renderStatsBar(contentContainer, filteredResults, activeTab.view.stats);
    }

    // Render interactive filters if configured
    if (activeTab.view.interactive_filters && activeTab.view.interactive_filters.length > 0) {
      this.renderInteractiveFilters(contentContainer, activeTab.view.interactive_filters, activeTab.results);
    }

    if (filteredResults.length === 0) {
      const emptyEl = document.createElement("div");
      emptyEl.className = "cm-query-embed-empty";
      emptyEl.textContent = activeTab.results.length === 0 ? "No results found" : "No results match selected filters";
      contentContainer.appendChild(emptyEl);
    } else {
      const viewType = activeTab.view.view_type;
      if (viewType === "Table") {
        this.renderTableInContainer(filteredResults, activeTab.view, contentContainer);
      } else if (viewType === "Kanban") {
        this.renderKanbanInContainer(filteredResults, activeTab.view, contentContainer);
      } else if (viewType === "Card") {
        this.renderCardInContainer(filteredResults, activeTab.view, contentContainer);
      } else {
        this.renderListInContainer(filteredResults, contentContainer);
      }
    }

    this.element.appendChild(contentContainer);
  }

  private renderTableInContainer(results: QueryResultItem[], view: QueryViewConfig, container: HTMLElement) {
    const table = document.createElement("table");
    table.className = "cm-query-embed-table";

    const columns = view.columns || [];

    // Check if we have both types
    const hasNotes = results.some(r => r.item_type === "note");
    const hasTasks = results.some(r => r.item_type === "task");
    const hasBoth = hasNotes && hasTasks;

    let displayColumns: string[];
    if (columns.length > 0) {
      // Add type column at the beginning for mixed results if not already specified
      displayColumns = hasBoth && !columns.includes("type")
        ? ["type", ...columns]
        : columns;
    } else if (hasBoth) {
      // Default columns for Both mode - columns that work for both types
      displayColumns = ["type", "title", "description", "priority", "due_date"];
    } else if (hasNotes) {
      displayColumns = ["title", "path"];
    } else {
      displayColumns = ["description", "priority", "context", "due_date", "note_title"];
    }

    // Header row
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");
    for (const col of displayColumns) {
      const th = document.createElement("th");
      th.textContent = this.formatColumnName(col);
      headerRow.appendChild(th);
    }
    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Body rows
    const tbody = document.createElement("tbody");
    for (const item of results) {
      const row = document.createElement("tr");
      row.className = item.item_type === "task" && item.task?.todo.completed
        ? "cm-query-row completed"
        : "cm-query-row";

      for (const col of displayColumns) {
        const td = document.createElement("td");
        td.innerHTML = this.getCellValue(item, col);

        // Make title/note_title/description columns clickable (for notes, description shows the title)
        const isClickableColumn = col === "note_title" || col === "title" ||
          (col === "description" && item.item_type === "note");
        if (isClickableColumn) {
          td.className = "cm-query-cell-link";
          const noteId = item.item_type === "task"
            ? item.task?.todo.note_id
            : item.note?.id;
          const notePath = item.item_type === "task"
            ? item.task?.note_path
            : item.note?.path;
          const noteTitle = item.item_type === "task"
            ? item.task?.note_title
            : item.note?.title;

          if (noteId && notePath) {
            td.onclick = (e) => {
              e.preventDefault();
              e.stopPropagation();
              workspaceStore.followLink({
                id: noteId,
                path: notePath,
                title: noteTitle ?? notePath.replace(".md", ""),
              });
            };
          }
        }

        row.appendChild(td);
      }
      tbody.appendChild(row);
    }
    table.appendChild(tbody);

    container.appendChild(table);
  }

  private renderListInContainer(results: QueryResultItem[], container: HTMLElement) {
    const list = document.createElement("ul");
    list.className = "cm-query-embed-list";

    for (const item of results) {
      const li = document.createElement("li");
      li.className = item.item_type === "task" && item.task?.todo.completed
        ? "cm-query-list-item completed"
        : "cm-query-list-item";

      if (item.item_type === "task" && item.task) {
        const checkbox = document.createElement("span");
        checkbox.className = "cm-query-checkbox";
        checkbox.textContent = item.task.todo.completed ? "☑" : "☐";
        li.appendChild(checkbox);

        const text = document.createElement("span");
        text.className = "cm-query-text";
        text.textContent = item.task.todo.description;
        li.appendChild(text);

        if (item.task.todo.priority) {
          const badge = document.createElement("span");
          badge.className = `cm-query-badge priority-${item.task.todo.priority}`;
          badge.textContent = item.task.todo.priority;
          li.appendChild(badge);
        }

        if (item.task.todo.context) {
          const badge = document.createElement("span");
          badge.className = "cm-query-badge context";
          badge.textContent = `@${item.task.todo.context}`;
          li.appendChild(badge);
        }

        const noteLink = document.createElement("button");
        noteLink.className = "cm-query-note-link";
        noteLink.textContent = item.task.note_title || item.task.note_path.replace(".md", "");
        noteLink.onclick = (e) => {
          e.preventDefault();
          e.stopPropagation();
          workspaceStore.followLink({
            id: item.task!.todo.note_id,
            path: item.task!.note_path,
            title: item.task!.note_title ?? item.task!.note_path.replace(".md", ""),
          });
        };
        li.appendChild(noteLink);
      } else if (item.note) {
        const noteLink = document.createElement("button");
        noteLink.className = "cm-query-note-link title";
        noteLink.textContent = item.note.title || item.note.path.replace(".md", "");
        noteLink.onclick = (e) => {
          e.preventDefault();
          e.stopPropagation();
          workspaceStore.followLink({
            id: item.note!.id,
            path: item.note!.path,
            title: item.note!.title ?? item.note!.path.replace(".md", ""),
          });
        };
        li.appendChild(noteLink);

        if (item.properties.length > 0) {
          const props = document.createElement("span");
          props.className = "cm-query-properties";
          props.textContent = item.properties
            .slice(0, 3)
            .map((p) => `${p.key}: ${p.value}`)
            .join(", ");
          li.appendChild(props);
        }
      }

      list.appendChild(li);
    }

    container.appendChild(list);
  }



  private renderCardInContainer(results: QueryResultItem[], view: QueryViewConfig, container: HTMLElement) {
    const cardConfig = view.card || {
      cover_property: null,
      display_fields: [],
      columns: 0,
    };

    const cardGrid = document.createElement("div");
    cardGrid.className = "cm-query-card-grid";

    for (const item of results) {
      const card = this.renderCardItem(item, cardConfig);
      cardGrid.appendChild(card);
    }

    container.appendChild(cardGrid);
  }

  private renderCardItem(item: QueryResultItem, config: CardConfig): HTMLElement {
    const card = document.createElement("button");
    card.className = "cm-query-card";
    
    const isCompleted = item.item_type === "task" && item.task?.todo.completed;
    if (isCompleted) {
      card.classList.add("completed");
    }

    // Get cover image
    const coverImage = this.getCoverImageForCard(item, config.cover_property);
    if (coverImage) {
      const coverEl = document.createElement("div");
      coverEl.className = "cm-query-card-cover";
      const img = document.createElement("img");
      img.src = coverImage;
      img.alt = "";
      img.loading = "lazy";
      coverEl.appendChild(img);
      card.appendChild(coverEl);
      card.classList.add("has-cover");
    }

    // Card content
    const content = document.createElement("div");
    content.className = "cm-query-card-content";

    // Title
    const title = document.createElement("div");
    title.className = "cm-query-card-title";
    title.textContent = this.getItemTitle(item);
    content.appendChild(title);

    // Display fields
    if (config.display_fields && config.display_fields.length > 0) {
      const fields = document.createElement("div");
      fields.className = "cm-query-card-fields";

      for (const field of config.display_fields) {
        const value = this.getFieldValue(item, field);
        if (value && field !== "description") {
          const fieldEl = document.createElement("div");
          fieldEl.className = "cm-query-card-field";

          const label = document.createElement("span");
          label.className = "cm-query-card-field-label";
          label.textContent = `${this.formatColumnName(field)}:`;
          fieldEl.appendChild(label);

          const valueEl = document.createElement("span");
          valueEl.className = "cm-query-card-field-value";
          if (field === "priority") {
            valueEl.classList.add(`priority-${value}`);
          }
          valueEl.textContent = value;
          fieldEl.appendChild(valueEl);

          fields.appendChild(fieldEl);
        }
      }

      if (fields.children.length > 0) {
        content.appendChild(fields);
      }
    }

    // Type badge
    const badge = document.createElement("div");
    badge.className = `cm-query-card-type-badge ${item.item_type}`;
    badge.textContent = item.item_type === "task" ? "Task" : "Note";
    content.appendChild(badge);

    card.appendChild(content);

    // Toggle button (if configured)
    if (config.toggle_property) {
      this.addToggleButton(card, item, config);
    }

    // Click handler
    card.onclick = (e) => {
      e.preventDefault();
      e.stopPropagation();
      const noteId = item.item_type === "task"
        ? item.task?.todo.note_id
        : item.note?.id;
      const notePath = item.item_type === "task"
        ? item.task?.note_path
        : item.note?.path;
      const noteTitle = item.item_type === "task"
        ? item.task?.note_title
        : item.note?.title;

      if (noteId && notePath) {
        workspaceStore.followLink({
          id: noteId,
          path: notePath,
          title: noteTitle ?? notePath.replace(".md", ""),
        });
      }
    };

    return card;
  }

  private getCoverImageForCard(item: QueryResultItem, coverProperty: string | null): string | null {
    if (!coverProperty) return null;
    
    const value = this.getFieldValue(item, coverProperty);
    if (!value) return null;

    // If it's already an absolute URL (http/https), return as-is
    if (value.startsWith("http://") || value.startsWith("https://")) {
      return value;
    }

    // Resolve relative path against vault root
    const vaultPath = vaultStore.info?.path;
    if (!vaultPath) return null;

    // Handle paths that might start with ./ or just be relative
    const cleanPath = value.startsWith("./") ? value.slice(2) : value;
    const fullPath = `${vaultPath}/${cleanPath}`;

    // Convert to Tauri asset URL
    return convertFileSrc(fullPath);
  }

  private getItemTitle(item: QueryResultItem): string {
    if (item.item_type === "task" && item.task) {
      return item.task.todo.description;
    }
    if (item.item_type === "note" && item.note) {
      return item.note.title || item.note.path.replace(".md", "");
    }
    return "Untitled";
  }

  private getFieldValue(item: QueryResultItem, field: string): string | null {
    // Check task fields first
    if (item.item_type === "task" && item.task) {
      const todo = item.task.todo;
      switch (field) {
        case "description":
          return todo.description;
        case "priority":
          return todo.priority;
        case "context":
          return todo.context;
        case "due_date":
          return todo.due_date;
        case "heading_path":
          return todo.heading_path;
      }
    }

    // Check note title
    if (item.item_type === "note" && item.note) {
      if (field === "title") {
        return item.note.title;
      }
      if (field === "description") {
        return item.note.title;
      }
    }

    // Check properties
    const prop = item.properties.find((p) => p.key === field);
    return prop?.value ?? null;
  }

  private renderKanbanInContainer(results: QueryResultItem[], view: QueryViewConfig, container: HTMLElement) {
    const kanbanConfig = view.kanban || {
      group_by: "priority",
      card_fields: ["description", "due_date"],
      show_uncategorized: true,
    };

    // Group results by the specified property
    const groups = this.groupResultsByProperty(results, kanbanConfig.group_by, kanbanConfig.show_uncategorized);

    // Create kanban board container
    const board = document.createElement("div");
    board.className = "cm-query-kanban-board";

    // Define column order for known properties
    const columnOrder = this.getColumnOrder(kanbanConfig.group_by, groups);

    // Render each column
    for (const columnName of columnOrder) {
      const columnItems = groups.get(columnName) || [];
      const column = this.renderKanbanColumn(columnName, columnItems, kanbanConfig);
      board.appendChild(column);
    }

    container.appendChild(board);
  }

  private groupResultsByProperty(
    results: QueryResultItem[],
    groupBy: string,
    showUncategorized: boolean
  ): Map<string, QueryResultItem[]> {
    const groups = new Map<string, QueryResultItem[]>();

    for (const item of results) {
      const value = this.getGroupValue(item, groupBy);

      if (!value && !showUncategorized) {
        continue;
      }

      const groupName = value || "Uncategorized";
      const existing = groups.get(groupName) || [];
      existing.push(item);
      groups.set(groupName, existing);
    }

    return groups;
  }

  private getGroupValue(item: QueryResultItem, groupBy: string): string | null {
    if (item.item_type === "task" && item.task) {
      switch (groupBy) {
        case "priority":
          return item.task.todo.priority;
        case "context":
          return item.task.todo.context;
        case "completed":
          return item.task.todo.completed ? "Completed" : "Not Completed";
        case "due_date":
          return this.categorizeDate(item.task.todo.due_date);
        default:
          // Check note properties
          const prop = item.properties.find((p) => p.key === groupBy);
          return prop?.value || null;
      }
    } else if (item.note) {
      const prop = item.properties.find((p) => p.key === groupBy);
      return prop?.value || null;
    }
    return null;
  }

  private categorizeDate(dateStr: string | null): string | null {
    if (!dateStr) return null;

    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const date = new Date(dateStr + "T00:00:00");

    if (date < today) return "Overdue";
    if (date.getTime() === today.getTime()) return "Today";

    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);
    if (date.getTime() === tomorrow.getTime()) return "Tomorrow";

    const weekEnd = new Date(today);
    weekEnd.setDate(weekEnd.getDate() + 7);
    if (date < weekEnd) return "This Week";

    return "Later";
  }

  private getColumnOrder(groupBy: string, groups: Map<string, QueryResultItem[]>): string[] {
    // Define preferred order for known group-by values
    const knownOrders: Record<string, string[]> = {
      priority: ["high", "medium", "low", "Uncategorized"],
      completed: ["Not Completed", "Completed"],
      due_date: ["Overdue", "Today", "Tomorrow", "This Week", "Later", "Uncategorized"],
    };

    const preferredOrder = knownOrders[groupBy];
    if (preferredOrder) {
      // Return columns in preferred order, but only if they exist
      const ordered: string[] = [];
      for (const col of preferredOrder) {
        if (groups.has(col)) {
          ordered.push(col);
        }
      }
      // Add any remaining columns not in the preferred order
      for (const col of groups.keys()) {
        if (!ordered.includes(col)) {
          ordered.push(col);
        }
      }
      return ordered;
    }

    // Default: sort alphabetically with "Uncategorized" at the end
    const columns = Array.from(groups.keys()).sort((a, b) => {
      if (a === "Uncategorized") return 1;
      if (b === "Uncategorized") return -1;
      return a.localeCompare(b);
    });
    return columns;
  }

  private renderKanbanColumn(
    columnName: string,
    items: QueryResultItem[],
    config: KanbanConfig
  ): HTMLElement {
    const column = document.createElement("div");
    column.className = "cm-query-kanban-column";

    // Column header
    const header = document.createElement("div");
    header.className = `cm-query-kanban-column-header ${this.getColumnColorClass(columnName, config.group_by)}`;

    const headerTitle = document.createElement("span");
    headerTitle.className = "cm-query-kanban-column-title";
    headerTitle.textContent = this.formatColumnName(columnName);
    header.appendChild(headerTitle);

    const headerCount = document.createElement("span");
    headerCount.className = "cm-query-kanban-column-count";
    headerCount.textContent = `${items.length}`;
    header.appendChild(headerCount);

    column.appendChild(header);

    // Column cards container
    const cardsContainer = document.createElement("div");
    cardsContainer.className = "cm-query-kanban-cards";

    for (const item of items) {
      const card = this.renderKanbanCard(item, config);
      cardsContainer.appendChild(card);
    }

    column.appendChild(cardsContainer);

    return column;
  }

  private getColumnColorClass(columnName: string, groupBy: string): string {
    if (groupBy === "priority") {
      switch (columnName.toLowerCase()) {
        case "high": return "priority-high";
        case "medium": return "priority-medium";
        case "low": return "priority-low";
      }
    }
    if (groupBy === "due_date") {
      switch (columnName) {
        case "Overdue": return "due-overdue";
        case "Today": return "due-today";
        case "Tomorrow": return "due-tomorrow";
      }
    }
    if (groupBy === "completed") {
      return columnName === "Completed" ? "status-done" : "status-pending";
    }
    return "";
  }

  private renderKanbanCard(item: QueryResultItem, config: KanbanConfig): HTMLElement {
    const card = document.createElement("div");
    const isCompleted = item.item_type === "task" && item.task?.todo.completed;
    card.className = `cm-query-kanban-card ${isCompleted ? "completed" : ""}`;

    // Card title/description
    const titleEl = document.createElement("div");
    titleEl.className = "cm-query-kanban-card-title";

    if (item.item_type === "task" && item.task) {
      titleEl.textContent = item.task.todo.description;
    } else if (item.note) {
      titleEl.textContent = item.note.title || item.note.path.replace(".md", "");
    }

    // Make title clickable
    titleEl.style.cursor = "pointer";
    titleEl.onclick = (e) => {
      e.preventDefault();
      e.stopPropagation();
      const noteId = item.item_type === "task"
        ? item.task?.todo.note_id
        : item.note?.id;
      const notePath = item.item_type === "task"
        ? item.task?.note_path
        : item.note?.path;
      const noteTitle = item.item_type === "task"
        ? item.task?.note_title
        : item.note?.title;

      if (noteId && notePath) {
        workspaceStore.followLink({
          id: noteId,
          path: notePath,
          title: noteTitle ?? notePath.replace(".md", ""),
        });
      }
    };

    card.appendChild(titleEl);

    // Card metadata based on config
    const metaEl = document.createElement("div");
    metaEl.className = "cm-query-kanban-card-meta";

    for (const field of config.card_fields) {
      const value = this.getCardFieldValue(item, field);
      if (value) {
        const fieldEl = document.createElement("span");
        fieldEl.className = `cm-query-kanban-card-field ${this.getFieldClass(field)}`;
        fieldEl.innerHTML = value;
        metaEl.appendChild(fieldEl);
      }
    }

    if (metaEl.children.length > 0) {
      card.appendChild(metaEl);
    }

    // Note link (if different from title)
    if (item.item_type === "task" && item.task) {
      const noteLink = document.createElement("div");
      noteLink.className = "cm-query-kanban-card-note";
      noteLink.textContent = item.task.note_title || item.task.note_path.replace(".md", "");
      noteLink.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        workspaceStore.followLink({
          id: item.task!.todo.note_id,
          path: item.task!.note_path,
          title: item.task!.note_title ?? item.task!.note_path.replace(".md", ""),
        });
      };
      card.appendChild(noteLink);
    }

    return card;
  }

  private getCardFieldValue(item: QueryResultItem, field: string): string | null {
    if (item.item_type === "task" && item.task) {
      switch (field) {
        case "description":
          return null; // Already shown as title
        case "priority":
          const p = item.task.todo.priority;
          return p ? `<span class="priority-badge priority-${p}">${p}</span>` : null;
        case "context":
          const c = item.task.todo.context;
          return c ? `@${c}` : null;
        case "due_date":
          return item.task.todo.due_date;
        case "note_title":
          return null; // Shown separately
        default:
          const prop = item.properties.find((pr) => pr.key === field);
          return prop?.value || null;
      }
    } else if (item.note) {
      const prop = item.properties.find((pr) => pr.key === field);
      return prop?.value || null;
    }
    return null;
  }

  private getFieldClass(field: string): string {
    switch (field) {
      case "priority": return "field-priority";
      case "context": return "field-context";
      case "due_date": return "field-due";
      default: return "field-default";
    }
  }



  private formatColumnName(col: string): string {
    return col
      .split("_")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(" ");
  }

  private getCellValue(item: QueryResultItem, col: string): string {
    // Handle "type" column for both types
    if (col === "type") {
      return item.item_type === "task"
        ? `<span class="cm-query-badge task-type">Task</span>`
        : `<span class="cm-query-badge note-type">Note</span>`;
    }

    if (item.item_type === "task" && item.task) {
      switch (col) {
        case "description":
          return item.task.todo.description;
        case "priority":
          const p = item.task.todo.priority;
          return p ? `<span class="cm-query-badge priority-${p}">${p}</span>` : "";
        case "context":
          const c = item.task.todo.context;
          return c ? `<span class="cm-query-badge context">@${c}</span>` : "";
        case "due_date":
          const d = item.task.todo.due_date;
          return d ? `<span class="cm-query-badge due-date">${d}</span>` : "";
        case "note_title":
          return item.task.note_title || item.task.note_path.replace(".md", "");
        case "title":
          // For "Both" mode, use note_title for tasks too
          return item.task.note_title || item.task.note_path.replace(".md", "");
        case "path":
          return item.task.note_path;
        case "completed":
          return item.task.todo.completed ? "Yes" : "No";
        default:
          // Check note properties
          const prop = item.properties.find((p) => p.key === col);
          return prop?.value || "";
      }
    } else if (item.note) {
      switch (col) {
        case "title":
          return item.note.title || item.note.path.replace(".md", "");
        case "path":
          return item.note.path;
        case "description":
          // Notes don't have description, show title instead in mixed view
          return item.note.title || item.note.path.replace(".md", "");
        case "note_title":
          // For "Both" mode, note_title is just the note's title
          return item.note.title || item.note.path.replace(".md", "");
        case "priority":
        case "context":
        case "due_date":
        case "completed":
          // Notes don't have these fields
          return "";
        default:
          // Check properties
          const prop = item.properties.find((p) => p.key === col);
          return prop?.value || "";
      }
    }
    return "";
  }

  ignoreEvent(): boolean {
    // Return true to prevent clicks from moving the cursor into the widget
    // This ensures tab clicks and other interactions don't trigger source view
    return true;
  }

  /**
   * Get active filter state for the current tab
   */
  private getActiveFilterState(): Map<string, Set<string>> {
    if (!this.tabFilterState.has(this.activeTabIndex)) {
      this.tabFilterState.set(this.activeTabIndex, new Map());
    }
    return this.tabFilterState.get(this.activeTabIndex)!;
  }

  /**
   * Extract unique values for a property from results
   */
  private getFilterValues(results: QueryResultItem[], key: string): string[] {
    const values = new Set<string>();
    for (const item of results) {
      const value = this.getFieldValue(item, key);
      if (value) {
        // Handle list properties (comma-separated)
        if (value.includes(',')) {
          value.split(',').forEach(v => values.add(v.trim()));
        } else {
          values.add(value);
        }
      }
    }
    return Array.from(values).sort();
  }

  /**
   * Apply active filters to results
   */
  private applyInteractiveFilters(results: QueryResultItem[]): QueryResultItem[] {
    const activeFilters = this.getActiveFilterState();
    if (activeFilters.size === 0) return results;

    return results.filter(item => {
      for (const [key, values] of activeFilters) {
        const propValue = this.getFieldValue(item, key) ?? '';
        const propValues = propValue.includes(',')
          ? propValue.split(',').map(v => v.trim())
          : [propValue];

        if (!propValues.some(v => values.has(v))) {
          return false;
        }
      }
      return true;
    });
  }

  /**
   * Toggle a filter value
   */
  private toggleFilter(key: string, value: string, multiSelect: boolean) {
    const activeFilters = this.getActiveFilterState();
    
    if (!activeFilters.has(key)) {
      activeFilters.set(key, new Set([value]));
    } else if (multiSelect) {
      const values = activeFilters.get(key)!;
      if (values.has(value)) {
        values.delete(value);
        if (values.size === 0) {
          activeFilters.delete(key);
        }
      } else {
        values.add(value);
      }
    } else {
      // Single select: toggle off if already selected, otherwise select
      const values = activeFilters.get(key)!;
      if (values.has(value) && values.size === 1) {
        activeFilters.delete(key);
      } else {
        activeFilters.set(key, new Set([value]));
      }
    }

    this.updateElement();
  }

  /**
   * Clear filter for a key
   */
  private clearFilter(key: string) {
    const activeFilters = this.getActiveFilterState();
    activeFilters.delete(key);
    this.updateElement();
  }

  /**
   * Create a filter chip element
   */
  private createFilterChip(label: string, isActive: boolean): HTMLButtonElement {
    const chip = document.createElement('button');
    chip.className = `cm-query-filter-chip ${isActive ? 'active' : ''}`;
    chip.textContent = label;
    return chip;
  }

  /**
   * Compute stats from results
   */
  private computeStats(results: QueryResultItem[], config: StatsConfig): { total: number; groups: Map<string, number> } {
    const stats = {
      total: results.length,
      groups: new Map<string, number>()
    };

    if (config.group_by) {
      for (const item of results) {
        const value = this.getFieldValue(item, config.group_by) ?? 'Uncategorized';
        stats.groups.set(value, (stats.groups.get(value) ?? 0) + 1);
      }
    }

    return stats;
  }

  /**
   * Render stats bar
   */
  private renderStatsBar(container: HTMLElement, results: QueryResultItem[], config: StatsConfig) {
    const stats = this.computeStats(results, config);
    const bar = document.createElement('div');
    bar.className = 'cm-query-stats-bar';

    if (config.total) {
      const totalStat = document.createElement('span');
      totalStat.className = 'cm-query-stat';
      totalStat.innerHTML = `<span class="cm-query-stat-num">${stats.total}</span> total`;
      bar.appendChild(totalStat);
    }

    for (const [value, count] of stats.groups) {
      const groupStat = document.createElement('span');
      groupStat.className = 'cm-query-stat';
      groupStat.innerHTML = `<span class="cm-query-stat-num">${count}</span> ${value}`;
      bar.appendChild(groupStat);
    }

    container.appendChild(bar);
  }

  /**
   * Render interactive filter chips
   */
  private renderInteractiveFilters(
    container: HTMLElement,
    filters: InteractiveFilter[],
    results: QueryResultItem[]
  ) {
    const filtersContainer = document.createElement('div');
    filtersContainer.className = 'cm-query-filters-container';

    for (const filter of filters) {
      const values = this.getFilterValues(results, filter.key);
      if (values.length === 0) continue;

      const filterRow = document.createElement('div');
      filterRow.className = 'cm-query-filter-row';

      // Filter label
      if (filter.label) {
        const label = document.createElement('span');
        label.className = 'cm-query-filter-label';
        label.textContent = filter.label + ':';
        filterRow.appendChild(label);
      }

      const chipsContainer = document.createElement('div');
      chipsContainer.className = 'cm-query-filter-chips';

      // "All" button
      if (filter.show_all) {
        const activeFilters = this.getActiveFilterState();
        const allBtn = this.createFilterChip('All', !activeFilters.has(filter.key));
        allBtn.onclick = (e) => {
          e.preventDefault();
          e.stopPropagation();
          this.clearFilter(filter.key);
        };
        chipsContainer.appendChild(allBtn);
      }

      // Value chips
      for (const value of values) {
        const activeFilters = this.getActiveFilterState();
        const isActive = activeFilters.get(filter.key)?.has(value) ?? false;
        const chip = this.createFilterChip(value, isActive);
        chip.onclick = (e) => {
          e.preventDefault();
          e.stopPropagation();
          this.toggleFilter(filter.key, value, filter.multi_select);
        };
        chipsContainer.appendChild(chip);
      }

      filterRow.appendChild(chipsContainer);
      filtersContainer.appendChild(filterRow);
    }

    container.appendChild(filtersContainer);
  }

  /**
   * Add toggle button to card
   */
  private addToggleButton(card: HTMLElement, item: QueryResultItem, config: CardConfig) {
    if (!config.toggle_property) return;

    const prop = item.properties.find(p => p.key === config.toggle_property);
    const isToggled = prop?.value === 'true';

    // Apply dim class if configured
    if (config.dim_when_true && isToggled) {
      card.classList.add('dimmed');
    }

    const btn = document.createElement('button');
    btn.className = `cm-query-toggle-btn ${isToggled ? 'toggled' : ''}`;
    btn.innerHTML = '✓';
    btn.onclick = async (e) => {
      e.preventDefault();
      e.stopPropagation();
      await this.toggleProperty(item, config.toggle_property!, !isToggled);
      
      // Update UI optimistically
      btn.classList.toggle('toggled');
      if (config.dim_when_true) {
        card.classList.toggle('dimmed');
      }
    };

    // Position the button
    const position = config.toggle_position || 'top-right';
    btn.style.position = 'absolute';
    if (position.includes('top')) {
      btn.style.top = '0.5rem';
    } else {
      btn.style.bottom = '0.5rem';
    }
    if (position.includes('right')) {
      btn.style.right = '0.5rem';
    } else {
      btn.style.left = '0.5rem';
    }

    card.appendChild(btn);
  }

  /**
   * Toggle a property on an item
   */
  private async toggleProperty(item: QueryResultItem, key: string, value: boolean): Promise<void> {
    const noteId = item.item_type === 'task'
      ? item.task!.todo.note_id
      : item.note!.id;

    try {
      await setProperty({
        note_id: noteId as any, // Type mismatch: binding says bigint but API accepts number
        key,
        value: value.toString(),
        property_type: 'boolean'
      });

      // Invalidate query cache to refresh results
      invalidateQueryCache();
    } catch (error) {
      console.error('Failed to toggle property:', error);
    }
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
 * Create decorations for query blocks
 * Uses line-by-line replacement to avoid CodeMirror's multi-line decoration restrictions
 */
function createDecorations(view: EditorView): DecorationSet {
  const blocks = findQueryBlocks(view.state);
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

    // Replace the opening ```query line with a hidden line widget
    const startLine = doc.line(block.startLine);
    allDecorations.push({
      from: startLine.from,
      to: startLine.to,
      decoration: Decoration.replace({ widget: hiddenWidget }),
      isLine: false,
    });

    // Hide all the intermediate YAML lines (one line at a time)
    for (let lineNum = block.startLine + 1; lineNum < block.endLine; lineNum++) {
      const line = doc.line(lineNum);
      allDecorations.push({
        from: line.from,
        to: line.to,
        decoration: Decoration.replace({ widget: new HiddenLineWidget() }),
        isLine: false,
      });
    }

    // Replace the closing ``` line with the results widget
    const endLine = doc.line(block.endLine);
    const widget = new QueryResultWidget(block);
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
 * ViewPlugin that manages query embed decorations
 */
const queryEmbedPlugin = ViewPlugin.fromClass(
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

// Inject query embed styles
const injectStyles = () => {
  if (typeof document === "undefined") return;
  if (document.getElementById("query-embed-extension-styles")) return;

  const style = document.createElement("style");
  style.id = "query-embed-extension-styles";
  style.textContent = `
    /* Hide the empty .cm-line elements for replaced lines */
    .cm-line:has(.cm-query-hidden-line) {
      display: none !important;
      height: 0 !important;
      min-height: 0 !important;
      padding: 0 !important;
      margin: 0 !important;
      line-height: 0 !important;
    }

    .cm-query-hidden-line {
      display: none;
    }

    .cm-query-embed {
      margin: 8px 0;
      border: 1px solid var(--border-default);
      border-radius: var(--radius-md);
      background: var(--bg-surface);
      overflow: hidden;
    }

    .cm-query-embed-header {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      padding: var(--spacing-2) var(--spacing-3);
      background: var(--bg-surface-sunken);
      border-bottom: 1px solid var(--border-default);
    }

    .cm-query-embed-header-tabs {
      padding: var(--spacing-2) var(--spacing-3);
    }

    .cm-query-embed-icon {
      font-size: 1.1em;
    }

    .cm-query-embed-title {
      font-weight: var(--font-weight-medium);
      font-size: var(--font-size-sm);
      color: var(--text-muted);
    }

    .cm-query-embed-loading,
    .cm-query-embed-empty {
      padding: var(--spacing-4);
      text-align: center;
      color: var(--text-muted);
      font-style: italic;
    }

    .cm-query-embed-error {
      padding: var(--spacing-3);
      color: var(--color-error);
      background: var(--color-error-light);
      border-radius: var(--radius-sm);
      margin: var(--spacing-2);
    }

    /* Tabs */
    .cm-query-tabs {
      display: flex;
      gap: var(--spacing-1);
      flex: 1;
      overflow-x: auto;
    }

    .cm-query-tab {
      padding: var(--spacing-1) var(--spacing-3);
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--radius-sm);
      color: var(--text-muted);
      font-size: var(--font-size-sm);
      font-weight: var(--font-weight-medium);
      cursor: pointer;
      transition: all 0.15s;
      white-space: nowrap;
    }

    .cm-query-tab:hover {
      background: var(--surface0);
      color: var(--text-primary);
    }

    .cm-query-tab.active {
      background: var(--mauve);
      color: var(--base);
      border-color: var(--mauve);
    }

    .cm-query-tab-content {
      /* Container for tab content */
    }

    /* Table view */
    .cm-query-embed-table {
      width: 100%;
      border-collapse: collapse;
      font-size: var(--font-size-sm);
    }

    .cm-query-embed-table th {
      text-align: left;
      padding: var(--spacing-2) var(--spacing-3);
      background: var(--bg-surface-sunken);
      border-bottom: 1px solid var(--border-default);
      color: var(--text-muted);
      font-weight: var(--font-weight-medium);
      text-transform: uppercase;
      font-size: var(--font-size-xs);
      letter-spacing: 0.5px;
    }

    .cm-query-embed-table td {
      padding: var(--spacing-2) var(--spacing-3);
      border-bottom: 1px solid var(--border-subtle);
    }

    .cm-query-row.completed td {
      color: var(--text-muted);
      text-decoration: line-through;
    }

    .cm-query-cell-link {
      cursor: pointer;
      color: var(--text-link);
    }

    .cm-query-cell-link:hover {
      text-decoration: underline;
    }

    /* List view */
    .cm-query-embed-list {
      list-style: none;
      margin: 0;
      padding: 0;
    }

    .cm-query-list-item {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      padding: var(--spacing-2) var(--spacing-3);
      border-bottom: 1px solid var(--border-subtle);
      flex-wrap: wrap;
    }

    .cm-query-list-item:last-child {
      border-bottom: none;
    }

    .cm-query-list-item.completed .cm-query-text {
      text-decoration: line-through;
      color: var(--text-muted);
    }

    .cm-query-checkbox {
      font-size: 1.1em;
    }

    .cm-query-text {
      flex: 1;
      min-width: 150px;
    }

    .cm-query-badge {
      font-size: var(--font-size-xs);
      padding: 1px 6px;
      border-radius: var(--radius-sm);
      background: var(--bg-tertiary);
      color: var(--text-muted);
    }

    .cm-query-badge.priority-high {
      background: var(--red);
      color: var(--base);
    }

    .cm-query-badge.priority-medium {
      background: var(--yellow);
      color: var(--crust);
    }

    .cm-query-badge.priority-low {
      background: var(--surface1);
      color: var(--text-muted);
    }

    .cm-query-badge.context {
      background: var(--blue);
      color: var(--base);
    }

    .cm-query-badge.due-date {
      background: var(--peach);
      color: var(--crust);
    }

    .cm-query-badge.task-type {
      background: var(--mauve);
      color: var(--base);
    }

    .cm-query-badge.note-type {
      background: var(--teal);
      color: var(--base);
    }

    .cm-query-note-link {
      background: none;
      border: none;
      color: var(--text-link);
      cursor: pointer;
      padding: 0;
      font-size: inherit;
      text-decoration: none;
    }

    .cm-query-note-link:hover {
      text-decoration: underline;
    }

    .cm-query-note-link.title {
      font-weight: var(--font-weight-medium);
    }

    .cm-query-properties {
      color: var(--text-muted);
      font-size: var(--font-size-xs);
      font-style: italic;
    }

    /* Kanban board styles */
    .cm-query-kanban-board {
      display: flex;
      gap: var(--spacing-3);
      overflow-x: auto;
      padding: var(--spacing-3);
      min-height: 200px;
    }

    .cm-query-kanban-column {
      flex: 0 0 240px;
      min-width: 240px;
      display: flex;
      flex-direction: column;
      background: var(--bg-surface-sunken);
      border-radius: var(--radius-md);
      max-height: 400px;
    }

    .cm-query-kanban-column-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: var(--spacing-2) var(--spacing-3);
      border-radius: var(--radius-md) var(--radius-md) 0 0;
      background: var(--surface1);
    }

    .cm-query-kanban-column-header.priority-high {
      background: var(--red);
      color: var(--base);
    }

    .cm-query-kanban-column-header.priority-medium {
      background: var(--yellow);
      color: var(--crust);
    }

    .cm-query-kanban-column-header.priority-low {
      background: var(--surface1);
    }

    .cm-query-kanban-column-header.due-overdue {
      background: var(--red);
      color: var(--base);
    }

    .cm-query-kanban-column-header.due-today {
      background: var(--peach);
      color: var(--crust);
    }

    .cm-query-kanban-column-header.due-tomorrow {
      background: var(--yellow);
      color: var(--crust);
    }

    .cm-query-kanban-column-header.status-done {
      background: var(--green);
      color: var(--base);
    }

    .cm-query-kanban-column-header.status-pending {
      background: var(--surface1);
    }

    .cm-query-kanban-column-title {
      font-weight: var(--font-weight-semibold);
      font-size: var(--font-size-sm);
      text-transform: capitalize;
    }

    .cm-query-kanban-column-count {
      font-size: var(--font-size-xs);
      opacity: 0.8;
      padding: 2px 6px;
      background: rgba(0, 0, 0, 0.1);
      border-radius: var(--radius-sm);
    }

    .cm-query-kanban-cards {
      flex: 1;
      overflow-y: auto;
      padding: var(--spacing-2);
      display: flex;
      flex-direction: column;
      gap: var(--spacing-2);
    }

    .cm-query-kanban-card {
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-md);
      padding: var(--spacing-2) var(--spacing-3);
      transition: box-shadow 0.15s, border-color 0.15s;
    }

    .cm-query-kanban-card:hover {
      border-color: var(--border-default);
      box-shadow: var(--shadow-sm);
    }

    .cm-query-kanban-card.completed {
      opacity: 0.6;
    }

    .cm-query-kanban-card.completed .cm-query-kanban-card-title {
      text-decoration: line-through;
      color: var(--text-muted);
    }

    .cm-query-kanban-card-title {
      font-size: var(--font-size-sm);
      font-weight: var(--font-weight-medium);
      color: var(--text-primary);
      margin-bottom: var(--spacing-1);
      line-height: 1.4;
    }

    .cm-query-kanban-card-title:hover {
      color: var(--text-link);
    }

    .cm-query-kanban-card-meta {
      display: flex;
      flex-wrap: wrap;
      gap: var(--spacing-1);
      font-size: var(--font-size-xs);
    }

    .cm-query-kanban-card-field {
      padding: 1px 4px;
      border-radius: var(--radius-sm);
      background: var(--bg-tertiary);
      color: var(--text-muted);
    }

    .cm-query-kanban-card-field.field-context {
      background: var(--blue);
      color: var(--base);
    }

    .cm-query-kanban-card-field.field-due {
      background: var(--peach);
      color: var(--crust);
    }

    .cm-query-kanban-card-field .priority-badge {
      padding: 1px 4px;
      border-radius: var(--radius-sm);
    }

    .cm-query-kanban-card-field .priority-badge.priority-high {
      background: var(--red);
      color: var(--base);
    }

    .cm-query-kanban-card-field .priority-badge.priority-medium {
      background: var(--yellow);
      color: var(--crust);
    }

    .cm-query-kanban-card-field .priority-badge.priority-low {
      background: var(--surface1);
      color: var(--text-muted);
    }

    .cm-query-kanban-card-note {
      margin-top: var(--spacing-2);
      font-size: var(--font-size-xs);
      color: var(--text-link);
      cursor: pointer;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .cm-query-kanban-card-note:hover {
      text-decoration: underline;
    }

    /* Stats Bar Styles */
    .cm-query-stats-bar {
      display: flex;
      gap: 1.5rem;
      padding: var(--spacing-3) var(--spacing-4);
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border-subtle);
    }

    .cm-query-stat {
      font-size: 0.9rem;
      color: var(--text-secondary);
    }

    .cm-query-stat-num {
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--mauve);
      margin-right: 0.25rem;
    }

    /* Interactive Filter Chips Styles */
    .cm-query-filters-container {
      padding: var(--spacing-2) var(--spacing-3);
      background: var(--bg-surface-sunken);
      border-bottom: 1px solid var(--border-subtle);
    }

    .cm-query-filter-row {
      display: flex;
      align-items: center;
      gap: var(--spacing-2);
      margin-bottom: var(--spacing-2);
    }

    .cm-query-filter-row:last-child {
      margin-bottom: 0;
    }

    .cm-query-filter-label {
      font-size: var(--font-size-sm);
      font-weight: var(--font-weight-medium);
      color: var(--text-secondary);
      min-width: 80px;
    }

    .cm-query-filter-chips {
      display: flex;
      flex-wrap: wrap;
      gap: var(--spacing-1);
      flex: 1;
    }

    .cm-query-filter-chip {
      padding: 0.25rem 0.75rem;
      border-radius: 1rem;
      background: var(--bg-surface);
      border: 1px solid var(--border-default);
      cursor: pointer;
      font-size: 0.85rem;
      transition: all 0.15s;
      color: var(--text-secondary);
      font-weight: var(--font-weight-medium);
    }

    .cm-query-filter-chip:hover {
      border-color: var(--color-primary);
      background: var(--bg-surface-raised);
    }

    .cm-query-filter-chip.active {
      background: var(--mauve);
      color: var(--base);
      border-color: var(--mauve);
    }

    /* Card Grid View Styles */
    .cm-query-card-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: var(--spacing-3);
      padding: var(--spacing-3);
    }

    .cm-query-card {
      display: flex;
      flex-direction: column;
      background: var(--bg-surface);
      border: 1px solid var(--border-default);
      border-radius: var(--radius-md);
      overflow: hidden;
      cursor: pointer;
      transition: transform 0.15s, box-shadow 0.15s, border-color 0.15s;
      text-align: left;
      padding: 0;
    }

    .cm-query-card:hover {
      transform: translateY(-2px);
      box-shadow: var(--shadow-lg);
      border-color: var(--color-primary);
    }

    .cm-query-card.completed {
      opacity: 0.6;
    }

    .cm-query-card-cover {
      width: 100%;
      height: 120px;
      overflow: hidden;
      background: var(--bg-surface-sunken);
    }

    .cm-query-card-cover img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    .cm-query-card-content {
      padding: var(--spacing-3);
      display: flex;
      flex-direction: column;
      gap: var(--spacing-2);
      flex: 1;
      position: relative;
    }

    .cm-query-card-title {
      font-size: var(--font-size-sm);
      font-weight: var(--font-weight-medium);
      color: var(--text-primary);
      line-height: 1.4;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }

    .cm-query-card-fields {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-1);
    }

    .cm-query-card-field {
      display: flex;
      gap: var(--spacing-1);
      font-size: var(--font-size-xs);
    }

    .cm-query-card-field-label {
      color: var(--text-muted);
      text-transform: capitalize;
    }

    .cm-query-card-field-value {
      color: var(--text-secondary);
    }

    .cm-query-card-field-value.priority-high {
      color: var(--red);
      font-weight: var(--font-weight-medium);
    }

    .cm-query-card-field-value.priority-medium {
      color: var(--yellow);
      font-weight: var(--font-weight-medium);
    }

    .cm-query-card-type-badge {
      position: absolute;
      top: var(--spacing-2);
      right: var(--spacing-2);
      font-size: 10px;
      padding: 2px 6px;
      border-radius: var(--radius-sm);
      font-weight: var(--font-weight-medium);
    }

    .cm-query-card-type-badge.task {
      background: var(--mauve);
      color: var(--base);
    }

    .cm-query-card-type-badge.note {
      background: var(--teal);
      color: var(--base);
    }

    /* Toggle Button Styles */
    .cm-query-toggle-btn {
      width: 24px;
      height: 24px;
      border-radius: 50%;
      border: 2px solid var(--border-default);
      background: transparent;
      color: transparent;
      cursor: pointer;
      transition: all 0.15s;
      z-index: 10;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 12px;
      font-weight: 600;
    }

    .cm-query-toggle-btn:hover {
      border-color: var(--mauve);
    }

    .cm-query-toggle-btn.toggled {
      background: var(--mauve);
      border-color: var(--mauve);
      color: var(--base);
    }

    .cm-query-card.dimmed {
      opacity: 0.5;
    }
  `;
  document.head.appendChild(style);
};

/**
 * Extension that provides query embed rendering.
 * Click navigation uses workspaceStore.followLink() directly.
 */
export function queryEmbedExtension() {
  injectStyles();
  return [queryEmbedPlugin];
}
