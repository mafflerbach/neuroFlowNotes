/**
 * Workspace store - manages the overall UI state (States A, B, C).
 *
 * State A: Calendar view without document (default)
 * State B: Calendar view with one open document
 * State C: Doc/Finder mode (no calendar, multi-column)
 */

import { getSetting, setSetting, type Theme } from "../services/settings";

export type WorkspaceState = "calendar-only" | "calendar-with-doc" | "doc-finder";
export type CalendarView = "monthly" | "weekly" | "daily";

interface OpenDoc {
  path: string;
  id: number;
  title: string | null;
}

interface PendingScroll {
  section: string; // slug to scroll to
  noteId: number;
}

interface OpenMedia {
  path: string;
  filename: string;
}

class WorkspaceStore {
  // Current workspace state
  state = $state<WorkspaceState>("calendar-only");

  // Calendar settings - initialized from saved settings
  calendarView = $state<CalendarView>(getSetting("defaultCalendarView"));
  selectedDate = $state<Date>(new Date());

  // Currently open media file (if any)
  openMedia = $state<OpenMedia | null>(null);

  // Pending section scroll (when navigating to [[note#section]])
  pendingScroll = $state<PendingScroll | null>(null);

  // Folder view visibility
  folderViewVisible = $state(true);

  // Doc list panel visibility
  docListVisible = $state(true);

  // Calendar/timeline visibility
  calendarVisible = $state(true);

  // Query builder view visibility
  queryViewVisible = $state(false);

  // Active plugin panel (null = closed, string = plugin panel id)
  activePluginPanel = $state<string | null>(null);

  // Computed: is any plugin panel visible
  get pluginPanelVisible(): boolean {
    return this.activePluginPanel !== null;
  }

  // Breadcrumb / document stack (for State C)
  breadcrumb = $state<OpenDoc[]>([]);

  // Settings
  multiColumnEditable = $state(true); // If false, non-active columns are read-only

  // Theme
  theme = $state<Theme>(getSetting("theme"));

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

  /** Toggle doc list panel visibility */
  toggleDocList() {
    this.docListVisible = !this.docListVisible;
  }

  /** Toggle calendar/timeline visibility */
  toggleCalendar() {
    this.calendarVisible = !this.calendarVisible;
  }

  /** Toggle query builder view */
  toggleQueryView() {
    this.queryViewVisible = !this.queryViewVisible;
  }

  /** Open query view */
  openQueryView() {
    this.queryViewVisible = true;
  }

  /** Close query view */
  closeQueryView() {
    this.queryViewVisible = false;
  }

  /** Toggle a specific plugin panel */
  togglePluginPanel(panelId: string) {
    if (this.activePluginPanel === panelId) {
      this.activePluginPanel = null;
    } else {
      this.activePluginPanel = panelId;
    }
  }

  /** Open a specific plugin panel */
  openPluginPanel(panelId: string) {
    this.activePluginPanel = panelId;
  }

  /** Close plugin panel */
  closePluginPanel() {
    this.activePluginPanel = null;
  }

  /** Check if a specific plugin panel is active */
  isPluginPanelActive(panelId: string): boolean {
    return this.activePluginPanel === panelId;
  }

  /** Switch calendar view (monthly, weekly, daily) */
  setCalendarView(view: CalendarView) {
    this.calendarView = view;
    // Switching to calendar view returns to State A or B
    if (this.state === "doc-finder") {
      this.state = this.breadcrumb.length > 0 ? "calendar-with-doc" : "calendar-only";
    }
  }

  /** Set the default calendar view (persisted to settings) */
  setDefaultCalendarView(view: CalendarView) {
    setSetting("defaultCalendarView", view);
  }

  /** Get the saved default calendar view */
  getDefaultCalendarView(): CalendarView {
    return getSetting("defaultCalendarView");
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
    // Close any open media file
    this.openMedia = null;

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

  /** Open a media file (image, audio, video) */
  openMediaFile(path: string) {
    const filename = path.split("/").pop() || path;

    // Close any open documents
    this.breadcrumb = [];
    this.openMedia = { path, filename };

    // Transition to calendar-with-doc state (reusing it for media)
    if (this.state === "calendar-only") {
      this.state = "calendar-with-doc";
    } else if (this.state === "doc-finder") {
      this.state = "calendar-with-doc";
    }
  }

  /** Close the currently open media file */
  closeMedia() {
    this.openMedia = null;
    if (this.breadcrumb.length === 0) {
      this.state = "calendar-only";
    }
  }

  /** Follow a [[wikilink]] - transitions to State C */
  followLink(doc: OpenDoc, section?: string) {
    // Transition to doc-finder mode
    this.state = "doc-finder";

    // Check if doc is already in breadcrumb
    const existingIndex = this.breadcrumb.findIndex((d) => d.path === doc.path);

    if (existingIndex >= 0) {
      // Doc already in breadcrumb - navigate to it (truncate after it)
      this.breadcrumb = this.breadcrumb.slice(0, existingIndex + 1);
    } else {
      // Add new doc to breadcrumb
      this.breadcrumb = [...this.breadcrumb, doc];
    }

    // Set pending scroll if section was specified
    if (section) {
      this.pendingScroll = { section, noteId: doc.id };
    }
  }

  /** Clear pending scroll (called after scrolling is complete) */
  clearPendingScroll() {
    this.pendingScroll = null;
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

  /** Update a document's path and title (after rename) */
  updateDocPath(oldPath: string, newPath: string, newTitle?: string) {
    this.breadcrumb = this.breadcrumb.map((doc) => {
      if (doc.path === oldPath) {
        return { ...doc, path: newPath, title: newTitle ?? doc.title };
      }
      return doc;
    });
  }

  /** Update paths for all documents inside a renamed folder */
  updateFolderPath(oldFolderPath: string, newFolderPath: string) {
    const oldPrefix = oldFolderPath + "/";
    const newPrefix = newFolderPath + "/";

    this.breadcrumb = this.breadcrumb.map((doc) => {
      if (doc.path.startsWith(oldPrefix)) {
        return { ...doc, path: doc.path.replace(oldPrefix, newPrefix) };
      }
      return doc;
    });
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

  // ========================================================================
  // Theme
  // ========================================================================

  /** Get the saved theme preference */
  getTheme(): Theme {
    return getSetting("theme");
  }

  /** Set the theme preference */
  setTheme(theme: Theme) {
    this.theme = theme;
    setSetting("theme", theme);
    this.applyTheme(theme);
  }

  /** Apply theme to document */
  applyTheme(theme: Theme) {
    const root = document.documentElement;

    if (theme === "system") {
      // Remove explicit theme, let CSS media query handle it
      root.removeAttribute("data-theme");
    } else {
      root.setAttribute("data-theme", theme);
    }
  }

  /** Initialize theme on app start */
  initTheme() {
    this.applyTheme(this.theme);
  }

  // ========================================================================
  // Vim Mode
  // ========================================================================

  // Vim mode state (read from settings)
  vimMode = $state(getSetting("vimMode"));

  /** Toggle vim mode */
  toggleVimMode() {
    this.vimMode = !this.vimMode;
    setSetting("vimMode", this.vimMode);
    // Note: Editor needs to be recreated to apply vim mode change
    // This will happen on next note load or reload
  }

  /** Set vim mode preference */
  setVimMode(enabled: boolean) {
    this.vimMode = enabled;
    setSetting("vimMode", enabled);
  }
}

export const workspaceStore = new WorkspaceStore();
