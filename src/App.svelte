<script lang="ts">
  import { onMount, onDestroy } from "svelte";
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
    saveNote,
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
    generateNoteFilename,
    generateNoteContent,
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

  // Workspace state
  const state = $derived(workspaceStore.state);
  const calendarView = $derived(workspaceStore.calendarView);
  const folderViewVisible = $derived(workspaceStore.folderViewVisible);
  const activeDoc = $derived(workspaceStore.activeDoc);
  const selectedDate = $derived(workspaceStore.selectedDate);

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
    if (!vaultStore.isOpen) {
      console.log("[App] fetchCalendarData: vault not open");
      return;
    }

    try {
      console.log("[App] fetchCalendarData: calendarView =", calendarView);
      // Always fetch DocList data for the selected date
      await fetchDocListData();

      if (calendarView === "weekly") {
        const { start, end } = getWeekRange(selectedDate);
        console.log("[App] Fetching weekly data:", start, "to", end);
        // Fetch schedule blocks for the week
        scheduleBlocks = await getScheduleBlocks(start, end);
        console.log("[App] Got schedule blocks:", scheduleBlocks.length);
        // Fetch notes for the week
        const notesData = await getNotesForDateRange(start, end);
        console.log("[App] Got notes for week:", notesData.length, "days");
        const weekMap = new Map<string, NoteListItem[]>();
        for (const [dateStr, notes] of notesData) {
          weekMap.set(dateStr, notes.map((n) => n.note));
        }
        notesForWeek = weekMap;
      } else if (calendarView === "monthly") {
        const { start, end } = getMonthRange(selectedDate);
        console.log("[App] Fetching monthly data:", start, "to", end);
        // Fetch notes for the month
        const notesData = await getNotesForDateRange(start, end);
        console.log("[App] Got notes for month:", notesData.length, "days");
        const monthMap = new Map<string, NoteListItem[]>();
        for (const [dateStr, notes] of notesData) {
          monthMap.set(dateStr, notes.map((n) => n.note));
        }
        notesForMonth = monthMap;
      } else if (calendarView === "daily") {
        const dateStr = formatDateKey(selectedDate);
        console.log("[App] Fetching daily data:", dateStr);
        // Fetch schedule blocks for the day
        scheduleBlocks = await getScheduleBlocks(dateStr, dateStr);
        console.log("[App] Got schedule blocks:", scheduleBlocks.length);
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
    // Explicitly read dependencies to track them
    const _view = calendarView;
    const _date = selectedDate;
    const isOpen = vaultStore.isOpen;

    if (isOpen) {
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
    console.log("[App] handleBlockClick:", block);

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
      // No note linked - create a new note and link it to this block
      try {
        const dateStr = block.date;
        const timeStr = block.start_time.slice(0, 5);
        const label = block.label || "note";
        const title = block.label || `Note for ${timeStr}`;
        const path = generateNoteFilename(dateStr, timeStr, label);
        const content = generateNoteContent(title);

        console.log("[App] Creating note for block:", path);
        const noteId = await saveNote(path, content);
        console.log("[App] Note created with id:", noteId);

        // Store metadata as properties in the database
        await setProperty({ note_id: noteId, key: "date", value: dateStr, property_type: "date" });
        await setProperty({ note_id: noteId, key: "time", value: timeStr, property_type: "time" });
        if (block.label) {
          await setProperty({ note_id: noteId, key: "title", value: block.label, property_type: "text" });
        }
        console.log("[App] Properties saved to database");

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
        });
        console.log("[App] Block updated with note_id");

        // Refresh data and open the note
        await vaultStore.refreshFolderTree();
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

  function handleBlockEdit(block: ScheduleBlockDto) {
    // Open the block for editing
    blockModalMode = "edit";
    blockModalDate = block.date;
    blockModalBlock = block;
    blockModalOpen = true;
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
  }

  async function handleSaveBlock(data: {
    date: string;
    start_time: string;
    end_time: string;
    label: string;
    color: string;
    context: string | null;
  }) {
    console.log("[App] handleSaveBlock called with:", data);
    try {
      if (blockModalMode === "create") {
        // Auto-create note file for the schedule block
        const timeStr = data.start_time.slice(0, 5);
        const path = generateNoteFilename(data.date, timeStr, data.label);
        const content = generateNoteContent(data.label);

        console.log("[App] Creating note for schedule block:", path);
        const noteId = await saveNote(path, content);
        console.log("[App] Note created with id:", noteId);

        // Store metadata as properties in the database
        await setProperty({ note_id: noteId, key: "date", value: data.date, property_type: "date" });
        await setProperty({ note_id: noteId, key: "time", value: timeStr, property_type: "time" });
        await setProperty({ note_id: noteId, key: "title", value: data.label, property_type: "text" });

        // Create schedule block linked to the note
        const request = {
          note_id: noteId,
          date: data.date,
          start_time: data.start_time,
          end_time: data.end_time,
          label: data.label,
          color: data.color,
          context: data.context,
        };
        console.log("[App] Creating schedule block with:", request);
        const blockId = await createScheduleBlock(request);
        console.log("[App] Schedule block created with id:", blockId);

        // Refresh folder tree to show the new file
        await vaultStore.refreshFolderTree();
      } else if (blockModalBlock) {
        const request = {
          id: blockModalBlock.id,
          note_id: blockModalBlock.note_id, // Preserve existing note link
          date: data.date,
          start_time: data.start_time,
          end_time: data.end_time,
          label: data.label,
          color: data.color,
          context: data.context,
        };
        console.log("[App] Updating schedule block with:", request);
        await updateScheduleBlock(request);
        console.log("[App] Schedule block updated");
      }
      handleCloseBlockModal();
      // Refresh calendar data
      console.log("[App] Refreshing calendar data...");
      await fetchCalendarData();
      console.log("[App] Calendar data refreshed");
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

  function handleLinkClick(targetPath: string) {
    // TODO: Resolve the link and open the target note
    console.log("Link clicked:", targetPath);
  }

  onMount(async () => {
    // Subscribe to backend events
    unlisteners.push(
      await onNotesUpdated((payload) => {
        console.log("Notes updated:", payload.note_ids);
        vaultStore.refreshFolderTree();
      })
    );

    unlisteners.push(
      await onNotesDeleted((payload) => {
        console.log("Notes deleted:", payload.note_ids);
        vaultStore.refreshFolderTree();
        // Close editor if current note was deleted
        if (activeDoc && payload.note_ids.includes(activeDoc.id)) {
          workspaceStore.closeDoc(activeDoc.path);
        }
      })
    );

    unlisteners.push(
      await onIndexComplete((payload) => {
        console.log(`Index complete: ${payload.notes_indexed} notes in ${payload.duration_ms}ms`);
        vaultStore.refresh();
        // Refresh calendar data after indexing
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
    {#if state === "calendar-only" || state === "calendar-with-doc"}
      <div class="doc-list-panel">
        <DocList
          {scheduledDocs}
          {journalDocs}
          {createdDocs}
          viewMode={calendarView}
          onDocClick={handleNoteClick}
        />
      </div>
    {/if}

    <!-- Main content area -->
    <main class="main-content">
      {#if state === "calendar-only"}
        <!-- State A: Calendar only -->
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
              onNoteClick={handleNoteClick}
              onEmptySlotClick={handleWeeklyEmptySlotClick}
            />
          {:else if calendarView === "daily"}
            <CalendarDaily
              {scheduleBlocks}
              onBlockClick={handleBlockClick}
              onBlockEdit={handleBlockEdit}
              onEmptySlotClick={handleEmptySlotClick}
            />
          {/if}
        </div>

      {:else if state === "calendar-with-doc"}
        <!-- State B: Calendar with one document -->
        <div class="split-view">
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
                onNoteClick={handleNoteClick}
                onEmptySlotClick={handleWeeklyEmptySlotClick}
              />
            {:else if calendarView === "daily"}
              <CalendarDaily
                {scheduleBlocks}
                onBlockClick={handleBlockClick}
                onBlockEdit={handleBlockEdit}
                onEmptySlotClick={handleEmptySlotClick}
              />
            {/if}
          </div>

          <div class="doc-panel">
            {#if activeDoc}
              <div class="doc-header">
                <span class="doc-title">{activeDoc.title || activeDoc.path}</span>
                <button
                  class="close-btn"
                  onclick={() => workspaceStore.closeDoc(activeDoc.path)}
                  title="Close document"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <div class="doc-content">
                <NoteEditor
                  noteId={activeDoc.id}
                  onLinkClick={handleLinkClick}
                />
              </div>
              <PropertiesPanel noteId={activeDoc.id} />
            {/if}
          </div>
        </div>

      {:else if state === "doc-finder"}
        <!-- State C: Doc-finder mode (multi-column) -->
        <div class="doc-finder-view">
          <Breadcrumb />
          <DocumentColumns onLinkClick={handleLinkClick} />
        </div>
      {/if}
    </main>
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
    onSave={handleSaveBlock}
    onDelete={handleDeleteBlock}
    onClose={handleCloseBlockModal}
  />
</div>

<style>
  :root {
    /* Colors */
    --primary-color: #4f6bed;
    --primary-hover: #3b5998;
    --primary-light-bg: #e0e7ff;
    --text-color: #1a1a1a;
    --text-muted: #666;
    --border-color: #e0e0e0;
    --border-light: #f0f0f0;
    --hover-bg: #f0f0f0;
    --active-bg: #e0e7ff;
    --active-color: #3b5998;
    --error-color: #d32f2f;
    --error-bg: #fee;
    --success-color: #2e7d32;
    --success-bg: #e8f5e9;

    /* Backgrounds */
    --topbar-bg: #fff;
    --sidebar-bg: #f8f9fa;
    --panel-bg: #fafafa;
    --surface-color: #ffffff;
    --calendar-bg: #fff;
    --editor-bg: #ffffff;
    --editor-header-bg: #fafafa;
    --editor-gutter-bg: #f5f5f5;
    --editor-gutter-color: #999;
    --editor-active-gutter-bg: #e8e8e8;
    --editor-active-line-bg: #f8f8f8;
    --editor-selection-bg: #c8daf8;
    --breadcrumb-bg: #f8f9fa;
    --column-header-bg: #f8f9fa;
    --active-column-header-bg: #e0e7ff;

    /* Calendar specific */
    --today-bg: #f0f7ff;
    --weekend-bg: #fafafa;
    --note-dot-color: #888;
    --nav-bg: #f0f0f0;
    --active-nav-bg: #fff;

    /* Modal */
    --modal-bg: #fff;
    --toggle-bg: #ccc;
    --input-bg: #fff;
    --btn-secondary-bg: #f0f0f0;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--editor-bg);
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
    border-right: 1px solid var(--border-color);
    overflow: hidden;
  }

  .doc-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .doc-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: var(--editor-header-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .doc-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-color);
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
    border-radius: 4px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--error-bg);
    color: var(--error-color);
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

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    :root {
      --primary-color: #6b8cff;
      --primary-hover: #5a7ae0;
      --primary-light-bg: #2d3748;
      --text-color: #e0e0e0;
      --text-muted: #999;
      --border-color: #333;
      --border-light: #2a2a2a;
      --hover-bg: #2a2a2a;
      --active-bg: #2d3748;
      --active-color: #90b0ff;
      --error-color: #ef5350;
      --error-bg: #3d2222;
      --success-color: #66bb6a;
      --success-bg: #1b3320;

      --topbar-bg: #1e1e1e;
      --sidebar-bg: #1a1a1a;
      --panel-bg: #1e1e1e;
      --surface-color: #2a2a2a;
      --calendar-bg: #1e1e1e;
      --editor-bg: #1e1e1e;
      --editor-header-bg: #252525;
      --editor-gutter-bg: #252525;
      --editor-gutter-color: #666;
      --editor-active-gutter-bg: #2a2a2a;
      --editor-active-line-bg: #252525;
      --editor-selection-bg: #264f78;
      --breadcrumb-bg: #252525;
      --column-header-bg: #252525;
      --active-column-header-bg: #2d3748;

      --today-bg: #1a2a3a;
      --weekend-bg: #1a1a1a;
      --note-dot-color: #666;
      --nav-bg: #2a2a2a;
      --active-nav-bg: #333;

      --modal-bg: #252525;
      --toggle-bg: #444;
      --input-bg: #1e1e1e;
      --btn-secondary-bg: #333;
    }
  }
</style>
