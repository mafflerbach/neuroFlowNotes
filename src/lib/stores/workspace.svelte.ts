/**
 * Workspace store - manages the overall UI state (States A, B, C).
 *
 * State A: Calendar view without document (default)
 * State B: Calendar view with one open document
 * State C: Doc/Finder mode (no calendar, multi-column)
 */

export type WorkspaceState = "calendar-only" | "calendar-with-doc" | "doc-finder";
export type CalendarView = "monthly" | "weekly" | "daily";

interface OpenDoc {
  path: string;
  id: number;
  title: string | null;
}

class WorkspaceStore {
  // Current workspace state
  state = $state<WorkspaceState>("calendar-only");

  // Calendar settings
  calendarView = $state<CalendarView>("weekly");
  selectedDate = $state<Date>(new Date());

  // Folder view visibility
  folderViewVisible = $state(true);

  // Breadcrumb / document stack (for State C)
  breadcrumb = $state<OpenDoc[]>([]);

  // Settings
  multiColumnEditable = $state(true); // If false, non-active columns are read-only

  // Computed: visible documents (last 3 of breadcrumb)
  get visibleDocs(): OpenDoc[] {
    return this.breadcrumb.slice(-3);
  }

  // Computed: active document (last in breadcrumb)
  get activeDoc(): OpenDoc | null {
    return this.breadcrumb.length > 0 ? this.breadcrumb[this.breadcrumb.length - 1] : null;
  }

  // ========================================================================
  // Actions
  // ========================================================================

  /** Toggle folder view visibility */
  toggleFolderView() {
    this.folderViewVisible = !this.folderViewVisible;
  }

  /** Switch calendar view (monthly, weekly, daily) */
  setCalendarView(view: CalendarView) {
    this.calendarView = view;
    // Switching to calendar view returns to State A or B
    if (this.state === "doc-finder") {
      this.state = this.breadcrumb.length > 0 ? "calendar-with-doc" : "calendar-only";
    }
  }

  /** Go to today */
  goToToday() {
    this.selectedDate = new Date();
    this.calendarView = "daily";
    if (this.state === "doc-finder") {
      this.state = this.breadcrumb.length > 0 ? "calendar-with-doc" : "calendar-only";
    }
  }

  /** Select a date in the calendar */
  selectDate(date: Date) {
    this.selectedDate = date;
  }

  /** Open a document from folder view or calendar */
  openDoc(doc: OpenDoc) {
    // If in State A, transition to State B
    if (this.state === "calendar-only") {
      this.state = "calendar-with-doc";
    }

    // If in State B (calendar mode), replace the single open doc
    if (this.state === "calendar-with-doc") {
      this.breadcrumb = [doc];
    }
    // If in State C, append to breadcrumb
    else if (this.state === "doc-finder") {
      // Don't add duplicates at the end
      if (this.activeDoc?.path !== doc.path) {
        this.breadcrumb = [...this.breadcrumb, doc];
      }
    }
  }

  /** Follow a [[wikilink]] - transitions to State C */
  followLink(doc: OpenDoc) {
    // Transition to doc-finder mode
    this.state = "doc-finder";

    // Add to breadcrumb if not already the active doc
    if (this.activeDoc?.path !== doc.path) {
      this.breadcrumb = [...this.breadcrumb, doc];
    }
  }

  /** Navigate breadcrumb - click on a breadcrumb item */
  navigateBreadcrumb(index: number) {
    // Truncate breadcrumb to clicked item (inclusive)
    this.breadcrumb = this.breadcrumb.slice(0, index + 1);
  }

  /** Close a document */
  closeDoc(path: string) {
    this.breadcrumb = this.breadcrumb.filter((d) => d.path !== path);

    // If no docs left, return to calendar-only
    if (this.breadcrumb.length === 0) {
      this.state = "calendar-only";
    }
  }

  /** Close all documents and return to calendar */
  closeAllDocs() {
    this.breadcrumb = [];
    this.state = "calendar-only";
  }

  /** Create new note */
  createNewNote(doc: OpenDoc) {
    // Open the new note
    this.openDoc(doc);
  }

  /** Return to calendar view (from doc-finder) */
  returnToCalendar() {
    if (this.state === "doc-finder") {
      this.state = this.breadcrumb.length > 0 ? "calendar-with-doc" : "calendar-only";
    }
  }
}

export const workspaceStore = new WorkspaceStore();
