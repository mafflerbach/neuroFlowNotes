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
    MediaViewer,
    QueryBuilder,
    Toast,
  } from "./lib/components";
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
  } from "./lib/services";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import type { NoteListItem, ScheduleBlockDto, NoteForDate } from "./lib/types";
  import { formatDateKey, getWeekRange, getMonthRange } from "./lib/utils/dateUtils";
  import {
    separateAndDeduplicateNotes,
    flattenNotesFromDateMap,
    generateNoteContent,
    replaceH1Title,
    generatePathFromTitle,
    titleToFilename,
    type DocWithSource,
  } from "./lib/utils/docListUtils";

  let unlisteners: UnlistenFn[] = [];
  let settingsOpen = $state(false);

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

  // Calendar data from backend
  let scheduleBlocks = $state<ScheduleBlockDto[]>([]);
  let notesForWeek = $state<Map<string, NoteListItem[]>>(new Map());
  let notesForMonth = $state<Map<string, NoteListItem[]>>(new Map());

  // DocList data for selected day
  let scheduledDocs = $state<DocWithSource[]>([]);
  let journalDocs = $state<DocWithSource[]>([]);
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
      // No note linked - check if a note with this path already exists, otherwise create one
      try {
        const dateStr = block.date;
        const timeStr = block.start_time.slice(0, 5);
        const title = block.label || `Note for ${timeStr}`;
        const path = titleToFilename(title);

        let noteId: number;

        // Try to get existing note first
        try {
          const existing = await getNoteContent(path);
          noteId = existing.id;
        } catch {
          // Note doesn't exist, create it
          const content = generateNoteContent(title);
          noteId = await saveNote(path, content);

          // Store metadata as properties in the database
          await setProperty({ note_id: noteId, key: "date", value: dateStr, property_type: "date" });
          await setProperty({ note_id: noteId, key: "time", value: timeStr, property_type: "time" });
          if (block.label) {
            await setProperty({ note_id: noteId, key: "title", value: block.label, property_type: "text" });
          }

          // Refresh folder tree to show the new file
          await vaultStore.refreshFolderTree();
        }

        // Update the block to link to this note
        await updateScheduleBlock({
          id: block.id,
          note_id: noteId,
          date: null,
          start_time: null,
          end_time: null,
          label: null,
          color: null,
          context: null,
          rrule: null,
        });

        // Refresh calendar data
        await fetchCalendarData();

        workspaceStore.openDoc({
          path,
          id: noteId,
          title,
        });
      } catch (e) {
        console.error("[App] Failed to create note for block:", e);
      }
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

  function handleDayClick(date: Date) {
    workspaceStore.selectDate(date);
    // Optionally switch to daily view
  }

  function handleQueryResultClick(noteId: number, notePath: string, noteTitle: string | null) {
    workspaceStore.closeQueryView();
    workspaceStore.openDoc({
      path: notePath,
      id: noteId,
      title: noteTitle,
    });
  }

  onMount(async () => {
    // Initialize theme from settings
    workspaceStore.initTheme();

    // Try to open the last used vault
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

<div class="app">
  <!-- Global Topbar -->
  <Topbar onOpenSettings={handleOpenSettings} />

  <div class="app-body">
    <!-- Sidebar (toggleable) -->
    {#if folderViewVisible}
      <Sidebar />
    {/if}

    <!-- DocList panel (between sidebar and calendar) -->
    {#if docListVisible && (workspaceState === "calendar-only" || workspaceState === "calendar-with-doc")}
      <div class="doc-list-panel">
        <DocList
          {scheduledDocs}
          {journalDocs}
          {createdDocs}
          onDocClick={handleNoteClick}
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
      <div class="query-panel">
        <QueryBuilder onResultClick={handleQueryResultClick} />
      </div>
    {/if}
  </div>

  <!-- Settings Modal -->
  <SettingsModal open={settingsOpen} onClose={handleCloseSettings} />

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

  .doc-list-panel {
    width: 220px;
    min-width: 180px;
    max-width: 280px;
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
    width: 400px;
    min-width: 320px;
    max-width: 500px;
    flex-shrink: 0;
    border-left: 1px solid var(--border-default);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
