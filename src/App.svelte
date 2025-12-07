<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { X } from "lucide-svelte";
  import "./lib/styles/theme.css";
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

  // Workspace state
  const workspaceState = $derived(workspaceStore.state);
  const calendarView = $derived(workspaceStore.calendarView);
  const folderViewVisible = $derived(workspaceStore.folderViewVisible);
  const docListVisible = $derived(workspaceStore.docListVisible);
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
        const title = block.label || `Note for ${timeStr}`;
        const path = titleToFilename(title);
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
        // Auto-create note file for the schedule block (filename based on title, like Obsidian)
        const timeStr = data.start_time.slice(0, 5);
        const path = titleToFilename(data.label);
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
        // If label changed and there's a linked note, sync the H1, title property, and rename file
        if (blockModalBlock.note_id && data.label !== blockModalBlock.label) {
          console.log("[App] Label changed, syncing to linked note...");
          try {
            const note = await getNote(blockModalBlock.note_id);
            const noteContent = await getNoteContent(note.path);
            const updatedContent = replaceH1Title(noteContent.content, data.label);
            await saveNote(note.path, updatedContent);

            // Rename the file to match the new label
            const newPath = generatePathFromTitle(note.path, data.label);
            if (newPath !== note.path) {
              console.log("[App] Renaming note:", note.path, "->", newPath);
              try {
                await renameNote(note.path, newPath);
                // Update workspace store's breadcrumb with new path
                workspaceStore.updateDocPath(note.path, newPath, data.label);
                console.log("[App] Note renamed successfully");
              } catch (e) {
                console.warn("[App] Could not rename file:", e);
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
            console.log("[App] Note H1 and title property synced");
          } catch (e) {
            console.error("[App] Failed to sync label to note:", e);
          }
        }

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

  onMount(async () => {
    // Initialize theme from settings
    workspaceStore.initTheme();

    // Try to open the last used vault
    await vaultStore.openLastVault();

    // Register callback for when editor updates schedule blocks
    editorStore.onScheduleBlocksUpdated = () => {
      console.log("[App] Schedule blocks updated, refreshing calendar data");
      fetchCalendarData();
    };

    // Subscribe to backend events
    unlisteners.push(
      await onNotesUpdated((payload) => {
        console.log("Notes updated:", payload.note_ids);
        vaultStore.refreshFolderTree();
        // Also refresh calendar data since note titles may have changed
        fetchCalendarData();
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

      {:else if workspaceState === "calendar-with-doc"}
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
</style>
