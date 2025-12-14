# Query Embeds

Query embeds allow you to display live query results directly within your notes. Inspired by Obsidian Bases, query embeds use YAML syntax to define filters and display options.

## Basic Syntax

Create a query embed by wrapping YAML configuration in a `query` code block:

````markdown
```query
filters:
  - key: project
    operator: Equals
    value: "MyProject"
result_type: Tasks
```
````

The query results will render inline when your cursor is outside the block. Move your cursor into the block to edit the YAML.

---

## Configuration Reference

### Top-Level Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `filters` | array | `[]` | List of property filter conditions |
| `match_mode` | string | `"All"` | How to combine filters: `"All"` (AND) or `"Any"` (OR) |
| `result_type` | string | `"Tasks"` | What to query: `"Tasks"`, `"Notes"`, or `"Both"` |
| `include_completed` | boolean | `false` | Include completed tasks in results |
| `limit` | number | `50` | Maximum number of results to display |
| `view` | object | (see below) | Display configuration |
| `tabs` | array | `[]` | Multi-tab configuration (see below) |

### Filter Properties

Each filter in the `filters` array has:

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `key` | string | Yes | Property name to filter on |
| `operator` | string | Yes | Comparison operator |
| `value` | string | No* | Value to compare against |

*Required for all operators except `Exists` and `NotExists`.

### Available Operators

| Operator | Description | Requires Value |
|----------|-------------|----------------|
| `Exists` | Property exists on the note | No |
| `NotExists` | Property does not exist | No |
| `Equals` | Exact match | Yes |
| `NotEquals` | Does not match | Yes |
| `Contains` | Contains substring | Yes |
| `StartsWith` | Starts with value | Yes |
| `EndsWith` | Ends with value | Yes |
| `ContainsAll` | List contains ALL values (comma-separated) | Yes |
| `ContainsAny` | List contains ANY value (comma-separated) | Yes |
| `DateOn` | Date equals value (YYYY-MM-DD) | Yes |
| `DateBefore` | Date is before value | Yes |
| `DateAfter` | Date is after value | Yes |
| `DateOnOrBefore` | Date is on or before value | Yes |
| `DateOnOrAfter` | Date is on or after value | Yes |

### View Configuration

The `view` object controls how results are displayed:

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `view_type` | string | `"Table"` | Display format: `"Table"`, `"List"`, or `"Kanban"` |
| `columns` | array | (auto) | Column names for table view |
| `sort` | object | `null` | Sort configuration |
| `kanban` | object | `null` | Kanban-specific configuration (when view_type is "Kanban") |

#### Sort Configuration

```yaml
sort:
  property: "due_date"
  direction: "Asc"  # or "Desc"
```

#### Kanban Configuration

When using `view_type: Kanban`, configure the board with:

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `group_by` | string | `"priority"` | Property to group cards into columns |
| `card_fields` | array | `["description", "due_date"]` | Fields to display on each card |
| `show_uncategorized` | boolean | `true` | Show cards without a value in "Uncategorized" column |

```yaml
view:
  view_type: Kanban
  kanban:
    group_by: "status"
    card_fields:
      - description
      - due_date
      - context
    show_uncategorized: true
```

#### Available Columns for Tasks

- `description` - Task text
- `priority` - Priority level (high/medium/low)
- `context` - Context tag (@work, @home, etc.)
- `due_date` - Due date
- `note_title` - Source note title
- `completed` - Completion status

#### Available Columns for Notes

- `title` - Note title
- `path` - File path
- Any property key from frontmatter

### Multi-Tab Queries

Use the `tabs` array to create multiple query tabs in a single embed. Each tab has its own filters, result type, and view configuration.

#### Tab Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `name` | string | (required) | Display name for the tab |
| `filters` | array | `[]` | Filters for this tab |
| `match_mode` | string | `"All"` | How to combine filters |
| `result_type` | string | `"Tasks"` | What to query |
| `include_completed` | boolean | `false` | Include completed tasks |
| `limit` | number | `50` | Maximum results |
| `view` | object | (default) | View configuration for this tab |

#### Multi-Tab Example

```query
tabs:
  - name: "High Priority"
    filters:
      - key: priority
        operator: Equals
        value: "high"
    result_type: Tasks
    view:
      view_type: List

  - name: "Due This Week"
    filters:
      - key: due_date
        operator: DateOnOrBefore
        value: "2025-12-20"
    result_type: Tasks
    view:
      view_type: Table
      columns:
        - description
        - due_date

  - name: "By Status"
    result_type: Tasks
    view:
      view_type: Kanban
      kanban:
        group_by: "status"
```

When `tabs` is present, the top-level `filters`, `result_type`, etc. are ignored.

---

## Examples

### Show All High Priority Tasks

```query
filters:
  - key: priority
    operator: Equals
    value: "high"
result_type: Tasks
include_completed: false
view:
  view_type: Table
  columns:
    - description
    - context
    - due_date
    - note_title
```

### Show Tasks from a Specific Project

```query
filters:
  - key: project
    operator: Equals
    value: "NeuroFlow"
result_type: Tasks
view:
  view_type: List
```

### Show Notes with a Specific Tag

```query
filters:
  - key: tags
    operator: Contains
    value: "meeting"
result_type: Notes
limit: 10
```

### Combine Multiple Filters (AND)

```query
filters:
  - key: project
    operator: Equals
    value: "NeuroFlow"
  - key: priority
    operator: Equals
    value: "high"
match_mode: All
result_type: Tasks
```

### Match Any Filter (OR)

```query
filters:
  - key: priority
    operator: Equals
    value: "high"
  - key: context
    operator: Equals
    value: "urgent"
match_mode: Any
result_type: Tasks
```

### Show Tasks Due Soon

```query
filters:
  - key: due_date
    operator: Exists
result_type: Tasks
include_completed: false
view:
  view_type: Table
  columns:
    - description
    - due_date
    - note_title
  sort:
    property: due_date
    direction: Asc
```

### Show Notes Missing a Property

```query
filters:
  - key: project
    operator: NotExists
result_type: Notes
limit: 20
```

### Simple Task List View

```query
result_type: Tasks
include_completed: false
limit: 25
view:
  view_type: List
```

### Tasks Due Before a Date

```query
filters:
  - key: due_date
    operator: DateBefore
    value: "2025-12-31"
result_type: Tasks
view:
  view_type: Table
  columns:
    - description
    - due_date
  sort:
    property: due_date
    direction: Asc
```

### Kanban Board by Priority

```query
result_type: Tasks
include_completed: false
view:
  view_type: Kanban
  kanban:
    group_by: "priority"
    card_fields:
      - description
      - context
      - due_date
```

### Kanban Board by Status

```query
filters:
  - key: project
    operator: Equals
    value: "NeuroFlow"
result_type: Tasks
view:
  view_type: Kanban
  kanban:
    group_by: "status"
    card_fields:
      - description
      - priority
```

### Notes with Specific Tags (ContainsAny)

```query
filters:
  - key: tags
    operator: ContainsAny
    value: "urgent,important"
result_type: Notes
```

---

## Behavior

### Live Updates

- Results refresh when you navigate away from and back to the note
- A 5-second cache prevents excessive API calls during editing
- Move cursor into the block to edit, out to see results

### Clicking Results

- Task and note titles are clickable
- Clicking opens the source note in the editor
- Works in both Table and List views

### Error Handling

If the YAML is invalid or the query fails, an error message is displayed in place of results.

---

## Tips

1. **Start Simple**: Begin with just `result_type: Tasks` and add filters as needed
2. **Use List View for Dashboards**: Compact list view works well for sidebar-style displays
3. **Limit Results**: Use `limit` to keep embeds performant with large vaults
4. **Check Property Names**: Use the Query Builder UI to discover available property keys
