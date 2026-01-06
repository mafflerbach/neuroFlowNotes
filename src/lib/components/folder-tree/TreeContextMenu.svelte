<script lang="ts">
  import { FilePlus, FolderPlus, Pencil, Trash2, Settings2, ChevronRight } from "lucide-svelte";
  import { listTemplates } from "../../services/api";

  interface Props {
    isDir: boolean;
    x: number;
    y: number;
    onNewFile?: () => void;
    onNewFromTemplate?: (templatePath: string) => void;
    onNewFolder?: () => void;
    onRename: () => void;
    onDelete: () => void;
    onProperties?: () => void;
    onClose: () => void;
  }

  let {
    isDir,
    x,
    y,
    onNewFile,
    onNewFromTemplate,
    onNewFolder,
    onRename,
    onDelete,
    onProperties,
    onClose,
  }: Props = $props();

  let templates = $state<string[]>([]);
  let templatesLoaded = $state(false);

  // Load templates when component mounts if it's a directory
  $effect(() => {
    if (isDir && !templatesLoaded) {
      loadTemplates();
    }
  });

  async function loadTemplates() {
    try {
      templates = await listTemplates();
      templatesLoaded = true;
    } catch (e) {
      console.error("Failed to load templates:", e);
      templates = [];
      templatesLoaded = true;
    }
  }

  function formatTemplateName(path: string): string {
    // "templates/ticket.md" → "ticket"
    // "templates/Sprint {{week}} ({{year}}).md" → "Sprint {{week}} ({{year}})"
    return path
      .replace(/^templates\//, "")
      .replace(/\.md$/, "");
  }

  function handleTemplateClick(templatePath: string) {
    onNewFromTemplate?.(templatePath);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<div
  class="context-menu"
  role="menu"
  tabindex="-1"
  style:left="{x}px"
  style:top="{y}px"
  onclick={(e) => e.stopPropagation()}
  onkeydown={handleKeydown}
>
  {#if isDir}
    <button class="menu-item" onclick={onNewFile}>
      <FilePlus size={14} />
      New Note
    </button>
    
    <!-- Template submenu -->
    <div class="menu-item-with-submenu">
      <div class="menu-item">
        <FilePlus size={14} />
        New Note from Template...
        <ChevronRight size={14} class="chevron" />
      </div>
      <div class="submenu">
        {#if templates.length > 0}
          {#each templates as template}
            <button class="submenu-item" onclick={() => handleTemplateClick(template)}>
              {formatTemplateName(template)}
            </button>
          {/each}
        {:else}
          <div class="submenu-empty">No templates found</div>
        {/if}
      </div>
    </div>
    
    <button class="menu-item" onclick={onNewFolder}>
      <FolderPlus size={14} />
      New Folder
    </button>
    <div class="menu-divider"></div>
  {/if}
  <button class="menu-item" onclick={onRename}>
    <Pencil size={14} />
    Rename
  </button>
  {#if isDir && onProperties}
    <button class="menu-item" onclick={onProperties}>
      <Settings2 size={14} />
      Properties
    </button>
  {/if}
  <button class="menu-item danger" onclick={onDelete}>
    <Trash2 size={14} />
    Delete
  </button>
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: var(--z-context-menu);
    min-width: 160px;
    background: var(--context-menu-bg);
    border: 1px solid var(--context-menu-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--context-menu-shadow);
    padding: var(--spacing-1);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
  }

  .menu-item:hover {
    background: var(--context-menu-item-hover-bg);
  }

  .menu-item.danger {
    color: var(--color-error);
  }

  .menu-item.danger:hover {
    background: var(--color-error-light);
  }

  .menu-divider {
    height: 1px;
    margin: var(--spacing-1) 0;
    background: var(--context-menu-separator);
  }

  /* Template submenu styles */
  .menu-item-with-submenu {
    position: relative;
  }

  .menu-item-with-submenu:hover .submenu {
    display: block;
  }

  .menu-item-with-submenu .menu-item {
    justify-content: space-between;
    cursor: default;
  }

  .menu-item-with-submenu .menu-item:hover {
    background: var(--context-menu-item-hover-bg);
  }

  .menu-item-with-submenu :global(.chevron) {
    margin-left: auto;
    opacity: 0.6;
  }

  .submenu {
    display: none;
    position: absolute;
    left: 100%;
    top: 0;
    min-width: 200px;
    max-height: 400px;
    overflow-y: auto;
    background: var(--context-menu-bg);
    border: 1px solid var(--context-menu-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--context-menu-shadow);
    padding: var(--spacing-1);
    z-index: calc(var(--z-context-menu) + 1);
  }

  .submenu-item {
    display: block;
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .submenu-item:hover {
    background: var(--context-menu-item-hover-bg);
  }

  .submenu-empty {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-style: italic;
  }
</style>
