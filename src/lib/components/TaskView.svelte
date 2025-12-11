<script lang="ts">
  import { queryTasks, getTaskContexts, toggleTodo } from "../services/api";
  import { workspaceStore } from "../stores";
  import type { TaskWithContext, TaskQuery } from "../types";

  // Filter state
  let showCompleted = $state(false);
  let selectedContext = $state<string | null>(null);
  let selectedPriority = $state<string | null>(null);
  let dueDateFilter = $state<"all" | "today" | "week" | "overdue">("all");

  // Data
  let tasks = $state<TaskWithContext[]>([]);
  let contexts = $state<string[]>([]);
  let loading = $state(true);

  // Build query from filters
  const buildQuery = (): TaskQuery => {
    const query: TaskQuery = {
      completed: showCompleted ? null : false,
      context: selectedContext,
      priority: selectedPriority,
      limit: 200,
    };

    const today = new Date().toISOString().split("T")[0];
    if (dueDateFilter === "today") {
      query.due_from = today;
      query.due_to = today;
    } else if (dueDateFilter === "week") {
      const weekEnd = new Date();
      weekEnd.setDate(weekEnd.getDate() + 7);
      query.due_from = today;
      query.due_to = weekEnd.toISOString().split("T")[0];
    } else if (dueDateFilter === "overdue") {
      query.due_to = new Date(Date.now() - 86400000).toISOString().split("T")[0];
    }

    return query;
  };

  // Load tasks
  async function loadTasks() {
    loading = true;
    try {
      const query = buildQuery();
      tasks = await queryTasks(query);
    } catch (e) {
      console.error("Failed to load tasks:", e);
      tasks = [];
    } finally {
      loading = false;
    }
  }

  // Load contexts
  async function loadContexts() {
    try {
      contexts = await getTaskContexts();
    } catch (e) {
      console.error("Failed to load contexts:", e);
    }
  }

  // Toggle task completion
  async function handleToggle(task: TaskWithContext) {
    const newCompleted = !task.todo.completed;
    await toggleTodo(task.todo.id, newCompleted);
    // Update local state immediately
    task.todo.completed = newCompleted;
    task.todo.completed_at = newCompleted ? new Date().toISOString() : null;
    // Reload to update filters
    if (!showCompleted && newCompleted) {
      await loadTasks();
    }
  }

  // Navigate to task's note
  function openTaskNote(task: TaskWithContext) {
    workspaceStore.followLink({
      path: task.note_path,
      id: task.todo.note_id,
      title: task.note_title ?? task.note_path.replace(".md", ""),
    });
  }

  // Get priority class
  function priorityClass(priority: string | null): string {
    if (!priority) return "";
    return `priority-${priority}`;
  }

  // Load on mount
  $effect(() => {
    loadTasks();
    loadContexts();
  });

  // Reload when filters change
  $effect(() => {
    // Access filter values to create dependency
    void showCompleted;
    void selectedContext;
    void selectedPriority;
    void dueDateFilter;
    loadTasks();
  });

  // Group tasks by due date
  const groupedTasks = $derived(() => {
    const groups = new Map<string, TaskWithContext[]>();
    const today = new Date().toISOString().split("T")[0];

    for (const task of tasks) {
      let key: string;
      if (!task.todo.due_date) {
        key = "No Due Date";
      } else if (task.todo.due_date < today) {
        key = "Overdue";
      } else if (task.todo.due_date === today) {
        key = "Today";
      } else {
        const tomorrow = new Date(Date.now() + 86400000).toISOString().split("T")[0];
        if (task.todo.due_date === tomorrow) {
          key = "Tomorrow";
        } else {
          key = task.todo.due_date;
        }
      }

      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(task);
    }

    // Sort groups: Overdue first, then Today, Tomorrow, dates, No Due Date last
    const order = ["Overdue", "Today", "Tomorrow"];
    const sortedEntries = [...groups.entries()].sort(([a], [b]) => {
      const aIdx = order.indexOf(a);
      const bIdx = order.indexOf(b);
      if (a === "No Due Date") return 1;
      if (b === "No Due Date") return -1;
      if (aIdx >= 0 && bIdx >= 0) return aIdx - bIdx;
      if (aIdx >= 0) return -1;
      if (bIdx >= 0) return 1;
      return a.localeCompare(b);
    });

    return new Map(sortedEntries);
  });
</script>

<div class="task-view">
  <div class="header">
    <h2>Task Inbox</h2>
    <span class="task-count">{tasks.filter(t => !t.todo.completed).length} tasks</span>
  </div>

  <div class="filters">
    <div class="filter-row">
      <label class="filter-item">
        <input type="checkbox" bind:checked={showCompleted} />
        <span>Show completed</span>
      </label>
    </div>

    <div class="filter-row">
      <select bind:value={dueDateFilter}>
        <option value="all">All dates</option>
        <option value="today">Due today</option>
        <option value="week">Due this week</option>
        <option value="overdue">Overdue</option>
      </select>

      <select bind:value={selectedPriority}>
        <option value={null}>All priorities</option>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
      </select>

      {#if contexts.length > 0}
        <select bind:value={selectedContext}>
          <option value={null}>All contexts</option>
          {#each contexts as ctx}
            <option value={ctx}>@{ctx}</option>
          {/each}
        </select>
      {/if}
    </div>
  </div>

  <div class="task-list">
    {#if loading}
      <div class="loading">Loading tasks...</div>
    {:else if tasks.length === 0}
      <div class="empty-state">No tasks match your filters</div>
    {:else}
      {#each [...groupedTasks().entries()] as [group, groupTasks] (group)}
        <div class="task-group">
          <h3 class="group-header" class:overdue={group === "Overdue"}>{group}</h3>
          <ul class="tasks">
            {#each groupTasks as task (task.todo.id)}
              <li class="task-item" class:completed={task.todo.completed}>
                <label class="task-checkbox">
                  <input
                    type="checkbox"
                    checked={task.todo.completed}
                    onchange={() => handleToggle(task)}
                  />
                </label>
                <div class="task-content">
                  <div class="task-main">
                    <span class="task-text">{task.todo.description}</span>
                    {#if task.todo.priority}
                      <span class="badge {priorityClass(task.todo.priority)}">{task.todo.priority}</span>
                    {/if}
                    {#if task.todo.context}
                      <span class="badge context">@{task.todo.context}</span>
                    {/if}
                  </div>
                  <div class="task-meta">
                    <button class="note-link" onclick={() => openTaskNote(task)}>
                      {task.note_title || task.note_path.replace(".md", "")}
                    </button>
                    {#if task.todo.heading_path}
                      <span class="heading-path"> &gt; {task.todo.heading_path}</span>
                    {/if}
                  </div>
                </div>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .task-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .header h2 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
  }

  .task-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .filters {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .filter-row {
    display: flex;
    gap: var(--spacing-2);
    flex-wrap: wrap;
    align-items: center;
  }

  .filter-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }

  select {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
  }

  .task-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2);
  }

  .loading,
  .empty-state {
    padding: var(--spacing-8);
    text-align: center;
    color: var(--text-muted);
  }

  .task-group {
    margin-bottom: var(--spacing-4);
  }

  .group-header {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-2) 0;
    padding: var(--spacing-1) var(--spacing-2);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .group-header.overdue {
    color: var(--red);
  }

  .tasks {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .task-item {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .task-item:hover {
    background: var(--bg-hover);
  }

  .task-item.completed .task-text {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  .task-checkbox {
    cursor: pointer;
    padding-top: 2px;
  }

  .task-content {
    flex: 1;
    min-width: 0;
  }

  .task-main {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    flex-wrap: wrap;
  }

  .task-text {
    font-size: var(--font-size-base);
  }

  .badge {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .badge.priority-high {
    background: var(--red);
    color: var(--base);
  }

  .badge.priority-medium {
    background: var(--yellow);
    color: var(--crust);
  }

  .badge.priority-low {
    background: var(--surface1);
    color: var(--text-muted);
  }

  .badge.context {
    background: var(--blue);
    color: var(--base);
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-top: var(--spacing-1);
  }

  .note-link {
    background: none;
    border: none;
    color: var(--text-link);
    cursor: pointer;
    padding: 0;
    font-size: inherit;
    text-decoration: underline;
    text-decoration-color: transparent;
    transition: text-decoration-color 0.15s;
  }

  .note-link:hover {
    text-decoration-color: var(--text-link);
  }

  .heading-path {
    color: var(--text-muted);
  }
</style>
