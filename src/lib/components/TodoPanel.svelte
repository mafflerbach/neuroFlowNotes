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
    background: var(--panel-bg, #fafafa);
    border-left: 1px solid var(--border-color, #e0e0e0);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .panel-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .task-count {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .empty-state {
    padding: 16px;
    text-align: center;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  .todo-group {
    margin-bottom: 16px;
  }

  .group-heading {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted, #666);
    margin: 0 0 8px 0;
    padding: 0 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .todo-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .todo-item {
    padding: 4px 8px;
    border-radius: 4px;
  }

  .todo-item:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .todo-item.completed .todo-text {
    text-decoration: line-through;
    color: var(--text-muted, #999);
  }

  .todo-label {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
  }

  .todo-label input[type="checkbox"] {
    margin-top: 2px;
    cursor: pointer;
  }

  .todo-text {
    flex: 1;
    line-height: 1.4;
  }
</style>
