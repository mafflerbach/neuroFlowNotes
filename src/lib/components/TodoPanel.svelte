<script lang="ts">
  import { editorStore } from "../stores";
  import type { TodoDto } from "../types";

  // Group todos by heading path
  const groupedTodos = $derived(() => {
    const groups = new Map<string, TodoDto[]>();

    for (const todo of editorStore.todos) {
      const key = todo.heading_path || "Uncategorized";
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(todo);
    }

    return groups;
  });

  async function handleToggle(todo: TodoDto) {
    await editorStore.toggleTodo(todo.id, !todo.completed);
  }
</script>

<div class="todo-panel">
  <div class="panel-header">
    <h3>Tasks</h3>
    <span class="task-count">
      {editorStore.todos.filter(t => !t.completed).length} remaining
    </span>
  </div>

  <div class="panel-content">
    {#if editorStore.todos.length === 0}
      <div class="empty-state">No tasks in this note</div>
    {:else}
      {#each [...groupedTodos().entries()] as [heading, todos] (heading)}
        <div class="todo-group">
          <h4 class="group-heading">{heading}</h4>
          <ul class="todo-list">
            {#each todos as todo (todo.id)}
              <li class="todo-item" class:completed={todo.completed}>
                <label class="todo-label">
                  <input
                    type="checkbox"
                    checked={todo.completed}
                    onchange={() => handleToggle(todo)}
                  />
                  <span class="todo-text">{todo.description}</span>
                </label>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .todo-panel {
    display: flex;
    flex-direction: column;
    width: 280px;
    min-width: 200px;
    max-width: 400px;
    height: 100%;
    background: var(--panel-bg);
    border-left: 1px solid var(--panel-border);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
  }

  .task-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2);
  }

  .empty-state {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-base);
  }

  .todo-group {
    margin-bottom: var(--spacing-4);
  }

  .group-heading {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-2) 0;
    padding: 0 var(--spacing-2);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .todo-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .todo-item {
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .todo-item:hover {
    background: var(--bg-hover);
  }

  .todo-item.completed .todo-text {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  .todo-label {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    cursor: pointer;
    font-size: var(--font-size-base);
  }

  .todo-label input[type="checkbox"] {
    margin-top: 2px;
    cursor: pointer;
  }

  .todo-text {
    flex: 1;
    line-height: var(--line-height-normal);
  }
</style>
