<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { X } from "lucide-svelte";
  import "./lib/styles/theme.css";
  // Import all theme files automatically
  import "./lib/styles/themes";
  import {
    Sidebar,
    Topbar,
    Breadcrumb,
    DocumentColumns,
    NoteEditor,
    CalendarMonthly,
    CalendarWeekly,
    CalendarDaily,
    DocList,
    SettingsModal,
    PropertiesPanel,
    ScheduleBlockModal,
    SearchModal,
    MediaViewer,
    QueryBuilder,
    Toast,
  } from "./lib/components";
  import { ResizeHandle } from "./lib/components/shared";
  import { vaultStore, editorStore, workspaceStore } from "./lib/stores";
  import {
    onNotesUpdated,
    onNotesDeleted,
    onIndexComplete,
    getScheduleBlocks,
    getNotesForDateRange,
    getNotesForDate,
    getNote,
    getNoteContent,
    saveNote,
    renameNote,
    setProperty,
    createScheduleBlock,
    updateScheduleBlock,
    deleteScheduleBlock,
    createDailyNote,
  } from "./lib/services";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import type { NoteListItem, ScheduleBlockDto, NoteForDate, EmbeddingSettings } from "./lib/types";
  import { DEFAULT_EMBEDDING_SETTINGS } from "./lib/types";
  import { formatDateKey, getWeekRange, getMonthRange } from "./lib/utils/dateUtils";
  import {
    separateAndDeduplicateNotes,
    flattenNotesFromDateMap,
    replaceH1Title,
    generatePathFromTitle,
    type DocWithSource,
  } from "./lib/utils/docListUtils";
  import { pluginRegistry, createBackendHooks } from "./lib/plugins";

  let unlisteners: UnlistenFn[] = [];
  let settingsOpen = $state(false);
  let searchOpen = $state(false);

  // Embedding settings (persisted in localStorage)
  let embeddingSettings = $state<EmbeddingSettings>(loadEmbeddingSettings());

  function loadEmbeddingSettings(): EmbeddingSettings {
    try {
      const stored = localStorage.getItem("neuroflow:embeddingSettings");
      if (stored) {
        return { ...DEFAULT_EMBEDDING_SETTINGS, ...JSON.parse(stored) };
      }
    } catch (e) {
      console.error("Failed to load embedding settings:", e);
    }
    return { ...DEFAULT_EMBEDDING_SETTINGS };
  }

  function saveEmbeddingSettings(settings: EmbeddingSettings) {
    embeddingSettings = settings;
    try {
      localStorage.setItem("neuroflow:embeddingSettings", JSON.stringify(settings));
    } catch (e) {
      console.error("Failed to save embedding settings:", e);
    }
  }

  // Schedule block modal state
  let blockModalOpen = $state(false);
  let blockModalMode = $state<"create" | "edit">("create");
  let blockModalDate = $state("");
  let blockModalHour = $state(9);
  let blockModalBlock = $state<ScheduleBlockDto | null>(null);
  let blockModalLinkedNote = $state<NoteListItem | null>(null);

  // Workspace state
  const workspaceState = $derived(workspaceStore.state);
  const calendarView = $derived(workspaceStore.calendarView);
  const folderViewVisible = $derived(workspaceStore.folderViewVisible);
  const docListVisible = $derived(workspaceStore.docListVisible);
  const calendarVisible = $derived(workspaceStore.calendarVisible);
  const activeDoc = $derived(workspaceStore.activeDoc);
  const selectedDate = $derived(workspaceStore.selectedDate);
  const openMedia = $derived(workspaceStore.openMedia);
  const queryViewVisible = $derived(workspaceStore.queryViewVisible);
  const pluginPanelVisible = $derived(workspaceStore.pluginPanelVisible);
  const activePluginPanelId = $derived(workspaceStore.activePluginPanel);

  // Get enabled plugin sidebar panels
  const pluginSidebarPanels = $derived(
    pluginRegistry.enabled
      .filter((p) => p.plugin.hooks?.sidebar?.panel)
      .map((p) => ({
        id: p.plugin.hooks!.sidebar!.panel!.id,
        label: p.plugin.hooks!.sidebar!.panel!.label,
        component: p.plugin.hooks!.sidebar!.panel!.component,
      }))
  );

  // Get the active plugin panel (if any)
  const activePluginPanel = $derived(
    pluginSidebarPanels.find((p) => p.id === activePluginPanelId)
  );

  // Calendar data from backend
  let scheduleBlocks = $state<ScheduleBlockDto[]>([]);
  let notesForWeek = $state<Map<string, NoteListItem[]>>(new Map());
  let notesForMonth = $state<Map<string, NoteListItem[]>>(new Map());

  // DocList data for selected day
  let scheduledDocs = $state<DocWithSource[]>([]);
  let journalDocs = $state<DocWithSource[]>([]);

  // Resizable panel widths (in pixels)
  let sidebarWidth = $state(250);
  let docListWidth = $state(220);
  let queryPanelWidth = $state(400);
  let pluginPanelWidth = $state(320);

  // Panel size constraints
  const SIDEBAR_MIN = 180;
  const SIDEBAR_MAX = 400;
  const DOCLIST_MIN = 150;
  const DOCLIST_MAX = 350;
  const QUERY_MIN = 300;
  const QUERY_MAX = 600;
  const PLUGIN_MIN = 250;
  const PLUGIN_MAX = 500;

  // Resize handlers
  function handleSidebarResize(delta: number) {
    sidebarWidth = Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, sidebarWidth + delta));
  }

  function handleDocListResize(delta: number) {
    docListWidth = Math.max(DOCLIST_MIN, Math.min(DOCLIST_MAX, docListWidth + delta));
  }

  function handleQueryPanelResize(delta: number) {
    // Query panel is on the right, so resize from the left edge (invert delta)
    queryPanelWidth = Math.max(QUERY_MIN, Math.min(QUERY_MAX, queryPanelWidth - delta));
  }

  function handlePluginPanelResize(delta: number) {
    // Plugin panel is on the right, so resize from the left edge (invert delta)
    pluginPanelWidth = Math.max(PLUGIN_MIN, Math.min(PLUGIN_MAX, pluginPanelWidth - delta));
  }
  let createdDocs = $state<DocWithSource[]>([]);

  // Fetch data for the current view
  async function fetchCalendarData() {
    if (!vaultStore.isOpen) return;

    try {
      // Always fetch DocList data for the selected date
      await fetchDocListData();

      if (calendarView === "weekly") {
        const { start, end } = getWeekRange(selectedDate);
        scheduleBlocks = await getScheduleBlocks(start, end);
        const notesData = await getNotesForDateRange(start, end);
        const weekMap = new Map<string, NoteListItem[]>();
        for (const [dateStr, notes] of notesData) {
          weekMap.set(dateStr, notes.map((n) => n.note));
        }
        notesForWeek = weekMap;
      } else if (calendarView === "monthly") {
        const { start, end } = getMonthRange(selectedDate);
        const notesData = await getNotesForDateRange(start, end);
        const monthMap = new Map<string, NoteListItem[]>();
        for (const [dateStr, notes] of notesData) {
          monthMap.set(dateStr, notes.map((n) => n.note));
        }
        notesForMonth = monthMap;
      } else if (calendarView === "daily") {
        const dateStr = formatDateKey(selectedDate);
        scheduleBlocks = await getScheduleBlocks(dateStr, dateStr);
      }
    } catch (e) {
      console.error("[App] Failed to fetch calendar data:", e);
    }
  }

  // Fetch data for the DocList component
  async function fetchDocListData() {
    if (!vaultStore.isOpen) return;

    try {
      let notes: NoteForDate[] = [];

      if (calendarView === "weekly") {
        const { start, end } = getWeekRange(selectedDate);
        const weekData = await getNotesForDateRange(start, end);
        notes = flattenNotesFromDateMap(weekData);
      } else if (calendarView === "monthly") {
        const { start, end } = getMonthRange(selectedDate);
        const monthData = await getNotesForDateRange(start, end);
        notes = flattenNotesFromDateMap(monthData);
      } else {
        const dateStr = formatDateKey(selectedDate);
        notes = await getNotesForDate(dateStr);
      }

      // Separate by source and deduplicate
      const separated = separateAndDeduplicateNotes(notes);
      scheduledDocs = separated.scheduledDocs;
      journalDocs = separated.journalDocs;
      createdDocs = separated.createdDocs;
    } catch (e) {
      console.error("Failed to fetch doc list data:", e);
    }
  }

  // Refetch when view or date changes
  $effect(() => {
    // Track dependencies by reading them
    void calendarView;
    void selectedDate;

    if (vaultStore.isOpen) {
      fetchCalendarData();
    }
  });

  // Track vault path to detect vault changes
  let lastVaultPath = $state<string | null>(null);

  // Reinitialize plugin system when vault opens/changes
  // Plugin configs are stored inside the vault, so we need to reload them
  $effect(() => {
    const currentPath = vaultStore.info?.path ?? null;

    if (vaultStore.isOpen && currentPath !== lastVaultPath) {
      lastVaultPath = currentPath;

      if (!pluginRegistry.isInitialized) {
        // First time initialization
        const backendHooks = createBackendHooks();
        pluginRegistry.initialize(backendHooks).catch((e) => {
          console.error("Failed to initialize plugin registry:", e);
        });
      } else {
        // Vault changed - reload configs
        pluginRegistry.reloadConfigs().catch((e) => {
          console.error("Failed to reload plugin configs:", e);
        });
      }
    }
  });

  // Load note content when activeDoc changes
  $effect(() => {
    const doc = activeDoc;
    if (doc) {
      editorStore.openNote(doc.path);
    } else {
      editorStore.close();
    }
  });

  
  function handleOpenSettings() {
    settingsOpen = true;
  }

  function handleCloseSettings() {
    settingsOpen = false;
  }

  async function handleBlockClick(block: ScheduleBlockDto) {
    if (block.note_id) {
      // Open existing linked note
      try {
        const note = await getNote(block.note_id);
        workspaceStore.openDoc({
          path: note.path,
          id: note.id,
          title: note.title,
        });
      } catch (e) {
        console.error("Failed to open note for block:", e);
      }
    } else {
      // No note linked - open the block for editing instead of auto-creating a note
      // This allows users to have blocks without notes, or to link an existing note
      handleBlockEdit(block);
    }
  }

  async function handleBlockEdit(block: ScheduleBlockDto) {
    // Open the block for editing
    blockModalMode = "edit";
    blockModalDate = block.date;
    blockModalBlock = block;

    // Fetch the linked note if one exists
    if (block.note_id) {
      try {
        const note = await getNote(block.note_id);
        blockModalLinkedNote = {
          id: note.id,
          path: note.path,
          title: note.title,
          pinned: note.pinned,
        };
      } catch {
        blockModalLinkedNote = null;
      }
    } else {
      blockModalLinkedNote = null;
    }

    blockModalOpen = true;
  }

  async function handleBlockMove(
    block: ScheduleBlockDto,
    newDate: string,
    newStartTime: string,
    newEndTime: string
  ) {
    try {
      await updateScheduleBlock({
        id: block.id,
        note_id: block.note_id, // Keep existing
        date: newDate,
        start_time: newStartTime,
        end_time: newEndTime,
        label: block.label, // Keep existing
        color: block.color, // Keep existing
        context: block.context, // Keep existing
        rrule: block.rrule, // Keep existing
      });

      // Refresh calendar data
      await fetchCalendarData();
    } catch (e) {
      console.error("[App] Failed to move block:", e);
    }
  }

  function handleNoteClick(note: NoteListItem) {
    workspaceStore.openDoc({
      path: note.path,
      id: note.id,
      title: note.title,
    });
  }

  function handleEmptySlotClick(hour: number) {
    // Open modal to create a new block for the selected date
    blockModalMode = "create";
    blockModalDate = formatDateKey(selectedDate);
    blockModalHour = hour;
    blockModalBlock = null;
    blockModalOpen = true;
  }

  function handleWeeklyEmptySlotClick(date: Date, hour: number) {
    // Open modal to create a new block for the clicked date
    blockModalMode = "create";
    blockModalDate = formatDateKey(date);
    blockModalHour = hour;
    blockModalBlock = null;
    blockModalOpen = true;
  }

  function handleCloseBlockModal() {
    blockModalOpen = false;
    blockModalBlock = null;
    blockModalLinkedNote = null;
  }

  async function handleSaveBlock(data: {
    date: string;
    start_time: string;
    end_time: string;
    label: string;
    color: string;
    context: string | null;
    rrule: string | null;
    note_id: number | null;
  }) {
    try {
      if (blockModalMode === "create") {
        // Create schedule block with optional note link.
        // If no note is linked, one will be created when clicking the block (see handleBlockClick).
        await createScheduleBlock({
          note_id: data.note_id,
          date: data.date,
          start_time: data.start_time,
          end_time: data.end_time,
          label: data.label,
          color: data.color,
          context: data.context,
          rrule: data.rrule,
        });
      } else if (blockModalBlock) {
        // If label changed and there's a linked note, sync the H1, title property, and rename file
        if (blockModalBlock.note_id && data.label !== blockModalBlock.label) {
          try {
            const note = await getNote(blockModalBlock.note_id);
            const noteContent = await getNoteContent(note.path);
            const updatedContent = replaceH1Title(noteContent.content, data.label);
            await saveNote(note.path, updatedContent);

            // Rename the file to match the new label
            const newPath = generatePathFromTitle(note.path, data.label);
            if (newPath !== note.path) {
              try {
                await renameNote(note.path, newPath);
                workspaceStore.updateDocPath(note.path, newPath, data.label);
              } catch (_) {
                // File might already exist with that name - continue silently
              }
            }

            // Update title property
            await setProperty({
              note_id: blockModalBlock.note_id,
              key: "title",
              value: data.label,
              property_type: "text",
            });

            // Refresh folder tree to show the renamed file
            await vaultStore.refreshFolderTree();
          } catch (e) {
            console.error("[App] Failed to sync label to note:", e);
          }
        }

        await updateScheduleBlock({
          id: blockModalBlock.id,
          note_id: data.note_id, // Use the note from the modal (may be changed or cleared)
          date: data.date,
          start_time: data.start_time,
          end_time: data.end_time,
          label: data.label,
          color: data.color,
          context: data.context,
          rrule: data.rrule,
        });
      }
      handleCloseBlockModal();
      await fetchCalendarData();
    } catch (e) {
      console.error("[App] Failed to save schedule block:", e);
    }
  }

  async function handleDeleteBlock() {
    if (!blockModalBlock) return;
    try {
      await deleteScheduleBlock(blockModalBlock.id);
      handleCloseBlockModal();
      // Refresh calendar data
      await fetchCalendarData();
    } catch (e) {
      console.error("Failed to delete schedule block:", e);
    }
  }

  async function handleDayClick(date: Date) {
    workspaceStore.selectDate(date);

    // Create or open the daily note for this date
    if (vaultStore.isOpen) {
      try {
        const dateStr = date.toISOString().split("T")[0];
        const result = await createDailyNote(dateStr);

        // Open the note in the editor
        workspaceStore.openDoc({
          path: result.path,
          id: result.id,
          title: result.title,
        });

        // Refresh folder tree if note was newly created
        if (result.created) {
          await vaultStore.refreshFolderTree();
        }
      } catch (e) {
        console.error("[App] Failed to create daily note:", e);
      }
    }
  }

  function handleQueryResultClick(noteId: number, notePath: string, noteTitle: string | null) {
    workspaceStore.closeQueryView();
    workspaceStore.openDoc({
      path: notePath,
      id: noteId,
      title: noteTitle,
    });
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Cmd+K / Ctrl+K to open search
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      searchOpen = true;
    }
  }

  onMount(async () => {
    // Initialize theme from settings
    workspaceStore.initTheme();

    // Try to open the last used vault
    // Plugin initialization happens automatically via $effect when vault opens
    await vaultStore.openLastVault();

    // Register callback for when editor updates schedule blocks
    editorStore.onScheduleBlocksUpdated = () => {
      fetchCalendarData();
    };

    // Subscribe to backend events
    unlisteners.push(
      await onNotesUpdated((_payload) => {
        vaultStore.refreshFolderTree();
        fetchCalendarData();
      })
    );

    unlisteners.push(
      await onNotesDeleted((payload) => {
        vaultStore.refreshFolderTree();
        if (activeDoc && payload.note_ids.includes(activeDoc.id)) {
          workspaceStore.closeDoc(activeDoc.path);
        }
      })
    );

    unlisteners.push(
      await onIndexComplete((_payload) => {
        vaultStore.refresh();
        fetchCalendarData();
      })
    );
  });

  onDestroy(() => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<SearchModal
  bind:open={searchOpen}
  onclose={() => (searchOpen = false)}
  embeddingSettings={embeddingSettings}
/>

<div class="app">
  <!-- Global Topbar -->
  <Topbar onOpenSettings={handleOpenSettings} />

  <div class="app-body">
    <!-- Sidebar (toggleable) -->
    {#if folderViewVisible}
      <div class="sidebar-wrapper" style:width="{sidebarWidth}px">
        <Sidebar />
        <ResizeHandle
          direction="horizontal"
          position="right"
          onResize={handleSidebarResize}
        />
      </div>
    {/if}

    <!-- DocList panel (between sidebar and calendar) -->
    {#if docListVisible && (workspaceState === "calendar-only" || workspaceState === "calendar-with-doc")}
      <div class="doc-list-panel" style:width="{docListWidth}px">
        <DocList
          {scheduledDocs}
          {journalDocs}
          {createdDocs}
          onDocClick={handleNoteClick}
        />
        <ResizeHandle
          direction="horizontal"
          position="right"
          onResize={handleDocListResize}
        />
      </div>
    {/if}

    <!-- Main content area -->
    <main class="main-content">
      {#if workspaceState === "calendar-only"}
        <!-- State A: Calendar only -->
        {#if calendarVisible}
          <div class="calendar-area">
            {#if calendarView === "monthly"}
              <CalendarMonthly
                {notesForMonth}
                onNoteClick={handleNoteClick}
                onDayClick={handleDayClick}
              />
            {:else if calendarView === "weekly"}
              <CalendarWeekly
                {scheduleBlocks}
                {notesForWeek}
                onBlockClick={handleBlockClick}
                onBlockEdit={handleBlockEdit}
                onBlockMove={handleBlockMove}
                onNoteClick={handleNoteClick}
                onEmptySlotClick={handleWeeklyEmptySlotClick}
              />
            {:else if calendarView === "daily"}
              <CalendarDaily
                {scheduleBlocks}
                onBlockClick={handleBlockClick}
                onBlockEdit={handleBlockEdit}
                onBlockMove={handleBlockMove}
                onEmptySlotClick={handleEmptySlotClick}
              />
            {/if}
          </div>
        {:else}
          <div class="empty-state">
            <p>Calendar hidden. Click the calendar icon to show.</p>
          </div>
        {/if}

      {:else if workspaceState === "calendar-with-doc"}
        <!-- State B: Calendar with one document -->
        <div class="split-view">
          {#if calendarVisible}
            <div class="calendar-panel">
              {#if calendarView === "monthly"}
                <CalendarMonthly
                  {notesForMonth}
                  onNoteClick={handleNoteClick}
                  onDayClick={handleDayClick}
                />
              {:else if calendarView === "weekly"}
                <CalendarWeekly
                  {scheduleBlocks}
                  {notesForWeek}
                  onBlockClick={handleBlockClick}
                  onBlockEdit={handleBlockEdit}
                  onBlockMove={handleBlockMove}
                  onNoteClick={handleNoteClick}
                  onEmptySlotClick={handleWeeklyEmptySlotClick}
                />
              {:else if calendarView === "daily"}
                <CalendarDaily
                  {scheduleBlocks}
                  onBlockClick={handleBlockClick}
                  onBlockEdit={handleBlockEdit}
                  onBlockMove={handleBlockMove}
                  onEmptySlotClick={handleEmptySlotClick}
                />
              {/if}
            </div>
          {/if}

          <div class="doc-panel" class:full-width={!calendarVisible}>
            {#if openMedia}
              <!-- Media Viewer -->
              <MediaViewer
                path={openMedia.path}
                vaultPath={vaultStore.info?.path || ""}
                onClose={() => workspaceStore.closeMedia()}
              />
            {:else if activeDoc}
              <div class="doc-header">
                <span class="doc-title">{activeDoc.title || activeDoc.path}</span>
                <button
                  class="close-btn"
                  onclick={() => workspaceStore.closeDoc(activeDoc.path)}
                  title="Close document"
                >
                  <X size={14} />
                </button>
              </div>
              <div class="doc-content">
                <NoteEditor />
              </div>
              <PropertiesPanel noteId={activeDoc.id} />
            {/if}
          </div>
        </div>

      {:else if workspaceState === "doc-finder"}
        <!-- State C: Doc-finder mode (multi-column) -->
        <div class="doc-finder-view">
          <Breadcrumb />
          <DocumentColumns />
        </div>
      {/if}
    </main>

    <!-- Query Builder Panel (slides in from right) -->
    {#if queryViewVisible}
      <div class="query-panel" style:width="{queryPanelWidth}px">
        <ResizeHandle
          direction="horizontal"
          position="left"
          onResize={handleQueryPanelResize}
        />
        <QueryBuilder onResultClick={handleQueryResultClick} />
      </div>
    {/if}

    <!-- Plugin Panel (slides in from right) -->
    {#if pluginPanelVisible && activePluginPanel}
      <div class="plugin-panel" style:width="{pluginPanelWidth}px">
        <ResizeHandle
          direction="horizontal"
          position="left"
          onResize={handlePluginPanelResize}
        />
        <activePluginPanel.component />
      </div>
    {:else if pluginPanelVisible && !activePluginPanel}
      <div class="plugin-panel empty-plugin-panel" style:width="{pluginPanelWidth}px">
        <ResizeHandle
          direction="horizontal"
          position="left"
          onResize={handlePluginPanelResize}
        />
        <p>Plugin not available.</p>
        <p class="hint">Enable this plugin in Settings → Plugins.</p>
      </div>
    {/if}
  </div>

  <!-- Settings Modal -->
  <SettingsModal
    open={settingsOpen}
    onClose={handleCloseSettings}
    embeddingSettings={embeddingSettings}
    onEmbeddingSettingsChange={saveEmbeddingSettings}
  />

  <!-- Schedule Block Modal -->
  <ScheduleBlockModal
    open={blockModalOpen}
    mode={blockModalMode}
    date={blockModalDate}
    initialHour={blockModalHour}
    block={blockModalBlock}
    linkedNote={blockModalLinkedNote}
    onSave={handleSaveBlock}
    onDelete={handleDeleteBlock}
    onClose={handleCloseBlockModal}
  />

  <!-- Global Toast Notifications -->
  <Toast />
</div>

<style>
  /* Theme variables are defined in /src/lib/styles/theme.css */

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-app);
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* Sidebar wrapper for resizing */
  .sidebar-wrapper {
    position: relative;
    flex-shrink: 0;
    display: flex;
    overflow: hidden;
  }

  .doc-list-panel {
    position: relative;
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .main-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* State A: Calendar only */
  .calendar-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* State B: Split view */
  .split-view {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .calendar-panel {
    flex: 0 0 50%;
    min-width: 300px;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-default);
    overflow: hidden;
  }

  .doc-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .doc-panel.full-width {
    max-width: none;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--font-size-md);
  }

  .doc-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2) var(--spacing-4);
    background: var(--column-header-bg);
    border-bottom: 1px solid var(--border-default);
  }

  .doc-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .doc-content {
    flex: 1;
    overflow: auto;
  }

  /* State C: Doc-finder mode */
  .doc-finder-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Query Builder Panel */
  .query-panel {
    position: relative;
    flex-shrink: 0;
    border-left: 1px solid var(--border-default);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Plugin Panel */
  .plugin-panel {
    position: relative;
    flex-shrink: 0;
    border-left: 1px solid var(--border-default);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .empty-plugin-panel {
    justify-content: center;
    align-items: center;
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
  }

  .empty-plugin-panel p {
    margin: 0;
  }

  .empty-plugin-panel .hint {
    font-size: var(--font-size-sm);
    margin-top: var(--spacing-2);
  }
</style>
