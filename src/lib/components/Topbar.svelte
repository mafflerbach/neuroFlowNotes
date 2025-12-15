<script lang="ts">
  import { Folder, List, Plus, Settings, Calendar, Search, Sparkles, FileText } from "lucide-svelte";
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
    if (!vaultStore.isOpen) return;

    const timestamp = Date.now();
    const path = `new-${timestamp}.md`;
    const content = `# New Note\n\n`;

    try {
      const noteId = await api.saveNote(path, content);
      await vaultStore.refreshFolderTree();
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
      <Folder size={18} />
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.docListVisible}
      onclick={() => workspaceStore.toggleDocList()}
      title="Toggle File List"
    >
      <List size={18} />
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.calendarVisible}
      onclick={() => workspaceStore.toggleCalendar()}
      title="Toggle Calendar"
    >
      <Calendar size={18} />
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.queryViewVisible}
      onclick={() => workspaceStore.toggleQueryView()}
      title="Query Builder"
      disabled={!vaultStore.isOpen}
    >
      <Search size={18} />
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.isPluginPanelActive("llm-daily-summarizer")}
      onclick={() => workspaceStore.togglePluginPanel("llm-daily-summarizer")}
      title="AI Daily Summary"
      disabled={!vaultStore.isOpen}
    >
      <Sparkles size={18} />
    </button>
    <button
      class="icon-btn"
      class:active={workspaceStore.isPluginPanelActive("llm-file-summarizer")}
      onclick={() => workspaceStore.togglePluginPanel("llm-file-summarizer")}
      title="AI File Summary"
      disabled={!vaultStore.isOpen}
    >
      <FileText size={18} />
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
      <Plus size={18} />
    </button>
  </div>

  <div class="topbar-right">
    <button
      class="icon-btn"
      onclick={onOpenSettings}
      title="Settings"
    >
      <Settings size={18} />
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--topbar-height);
    padding: 0 var(--spacing-3);
    background: var(--topbar-bg);
    border-bottom: 1px solid var(--topbar-border);
    gap: var(--spacing-4);
  }

  .topbar-left,
  .topbar-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .topbar-center {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
  }

  .calendar-nav {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-lg);
    padding: var(--spacing-1);
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    cursor: pointer;
    transition: background var(--transition-normal), color var(--transition-normal);
  }

  .nav-btn:hover {
    background: var(--bg-hover);
  }

  .nav-btn.active {
    background: var(--bg-surface);
    color: var(--color-primary);
    box-shadow: var(--shadow-md);
  }

  .today-btn .dot {
    width: 6px;
    height: 6px;
    background: var(--color-primary);
    border-radius: var(--radius-full);
  }

  .divider {
    width: 1px;
    height: 24px;
    background: var(--border-default);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: var(--radius-lg);
    color: var(--text-primary);
    cursor: pointer;
    transition: background var(--transition-normal);
  }

  .icon-btn:hover {
    background: var(--bg-hover);
  }

  .icon-btn.active {
    background: var(--bg-active);
    color: var(--color-primary);
  }

  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
