<script lang="ts">
  import { workspaceStore } from "../stores/workspace.svelte";
  import { vaultStore } from "../stores/vault.svelte";
  import * as api from "../services/api";

  interface Props {
    onOpenSettings?: () => void;
  }

  let { onOpenSettings }: Props = $props();

  const isMonthly = $derived(workspaceStore.calendarView === "monthly");
  const isWeekly = $derived(workspaceStore.calendarView === "weekly");
  const isDaily = $derived(workspaceStore.calendarView === "daily");

  async function handleNewNote() {
    if (!vaultStore.isOpen) {
      console.log("[Topbar] handleNewNote: vault not open");
      return;
    }

    // Generate a new note path
    const timestamp = Date.now();
    const path = `new-${timestamp}.md`;
    const content = `# New Note\n\n`;

    try {
      console.log("[Topbar] Creating new note:", path);
      const noteId = await api.saveNote(path, content);
      console.log("[Topbar] Note created with id:", noteId);

      console.log("[Topbar] Refreshing folder tree...");
      await vaultStore.refreshFolderTree();
      console.log("[Topbar] Folder tree refreshed");

      workspaceStore.openDoc({
        path,
        id: noteId,
        title: "New Note",
      });
    } catch (e) {
      console.error("[Topbar] Failed to create new note:", e);
    }
  }
</script>

<header class="topbar">
  <div class="topbar-left">
    <button
      class="icon-btn"
      class:active={workspaceStore.folderViewVisible}
      onclick={() => workspaceStore.toggleFolderView()}
      title="Toggle Folder View"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.docListVisible}
      onclick={() => workspaceStore.toggleDocList()}
      title="Toggle File List"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="8" y1="6" x2="21" y2="6" />
        <line x1="8" y1="12" x2="21" y2="12" />
        <line x1="8" y1="18" x2="21" y2="18" />
        <line x1="3" y1="6" x2="3.01" y2="6" />
        <line x1="3" y1="12" x2="3.01" y2="12" />
        <line x1="3" y1="18" x2="3.01" y2="18" />
      </svg>
    </button>
  </div>

  <div class="topbar-center">
    <div class="calendar-nav">
      <button
        class="nav-btn"
        class:active={isMonthly}
        onclick={() => workspaceStore.setCalendarView("monthly")}
        title="Monthly View"
      >
        M
      </button>
      <button
        class="nav-btn"
        class:active={isWeekly}
        onclick={() => workspaceStore.setCalendarView("weekly")}
        title="Weekly View"
      >
        W
      </button>
      <button
        class="nav-btn today-btn"
        class:active={isDaily}
        onclick={() => workspaceStore.goToToday()}
        title="Today"
      >
        <span class="dot"></span>
        Today
      </button>
    </div>

    <div class="divider"></div>

    <button
      class="icon-btn"
      onclick={handleNewNote}
      title="New Note"
      disabled={!vaultStore.isOpen}
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
    </button>
  </div>

  <div class="topbar-right">
    <button
      class="icon-btn"
      onclick={onOpenSettings}
      title="Settings"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 12px;
    background: var(--topbar-bg, #fff);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    gap: 16px;
  }

  .topbar-left,
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-center {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .calendar-nav {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--nav-bg, #f0f0f0);
    border-radius: 8px;
    padding: 4px;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border: none;
    background: transparent;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-color, #333);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .nav-btn:hover {
    background: var(--hover-bg, #e0e0e0);
  }

  .nav-btn.active {
    background: var(--active-nav-bg, #fff);
    color: var(--primary-color, #4f6bed);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .today-btn .dot {
    width: 6px;
    height: 6px;
    background: var(--primary-color, #4f6bed);
    border-radius: 50%;
  }

  .divider {
    width: 1px;
    height: 24px;
    background: var(--border-color, #e0e0e0);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: 8px;
    color: var(--text-color, #333);
    cursor: pointer;
    transition: background 0.15s;
  }

  .icon-btn:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .icon-btn.active {
    background: var(--active-bg, #e0e7ff);
    color: var(--primary-color, #4f6bed);
  }

  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
