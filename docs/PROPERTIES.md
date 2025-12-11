# Properties

Properties (also known as frontmatter or metadata) let you add structured data to your notes. NeuroFlow Notes supports YAML frontmatter at the top of your Markdown files, plus a visual editor for easy management.

## Basic Syntax

Add properties to any note using YAML frontmatter at the very beginning of the file:

```markdown
---
project: NeuroFlow
status: active
priority: high
due: 2024-12-31
---

# My Note Title

Content goes here...
```

The frontmatter block must:
- Start at the very first line of the file
- Begin and end with `---` on their own lines
- Contain valid YAML between the delimiters

---

## Property Types

NeuroFlow Notes supports several property types:

| Type | Example | Description |
|------|---------|-------------|
| Text | `status: active` | Simple text value |
| Number | `count: 42` | Numeric value |
| Date | `due: 2024-12-31` | Date in YYYY-MM-DD format |
| Checkbox | `completed: true` | Boolean true/false |
| List | `tags: [a, b, c]` | Array of values |

### Text Properties

```yaml
---
project: NeuroFlow
author: Jane Doe
status: draft
---
```

### Number Properties

```yaml
---
word_count: 1500
version: 2
rating: 4.5
---
```

### Date Properties

```yaml
---
created: 2024-01-15
due: 2024-12-31
reviewed: 2024-06-20
---
```

### Checkbox Properties

```yaml
---
published: true
archived: false
featured: true
---
```

### List Properties

```yaml
---
tags:
  - rust
  - development
  - tauri
collaborators:
  - Alice
  - Bob
---
```

Or inline format:

```yaml
---
tags: [rust, development, tauri]
---
```

---

## Visual Property Editor

You can edit properties without touching the YAML using the Properties Panel in the sidebar:

1. Open a note
2. Click the **Properties** tab in the right sidebar
3. View existing properties or add new ones
4. Edit values inline
5. Changes are automatically saved to the frontmatter

### Adding a Property

1. Click **+ Add Property** in the Properties Panel
2. Enter a key name
3. Enter a value
4. Optionally select a property type

### Editing a Property

1. Click on the property value in the Properties Panel
2. Edit the value inline
3. Press Enter or click away to save

### Deleting a Property

1. Hover over the property in the Properties Panel
2. Click the delete (X) icon
3. Confirm the deletion

---

## Properties Editor (Settings)

For bulk management of properties across your entire vault, use the Properties Editor in Settings:

**Settings > Properties**

### Three-Column Layout

The Properties Editor shows:

1. **Keys Column** - All property keys used in your vault with usage counts
2. **Values Column** - All values for the selected key with usage counts
3. **Notes Column** - Notes using the selected key/value

### Rename a Property Key

Use this to fix typos or normalize naming (e.g., "Project" vs "project"):

1. Open Settings > Properties
2. Find the key in the Keys column
3. Click the pencil icon
4. Enter the new name
5. Press Enter to save

All notes with that property will be updated.

### Rename a Property Value

Use this to normalize values (e.g., "In Progress" vs "in-progress"):

1. Select a key in the Keys column
2. Find the value in the Values column
3. Click the pencil icon
4. Enter the new value
5. Press Enter to save

All notes with that key-value pair will be updated.

### Merge Property Keys

Use this to combine two property keys that represent the same concept:

1. Click **Merge** in the Keys column header
2. Click the key to merge FROM (source - will be deleted)
3. Click the key to merge INTO (target - will remain)
4. Click **Merge** to confirm

All values from the source key will be moved to the target key. Notes that have both keys will have the source key removed.

### Delete a Property Key

Use this to remove a property from all notes:

1. Find the key in the Keys column
2. Click the X icon
3. Confirm the deletion

The property will be removed from all notes in your vault.

---

## Using Properties in Queries

Properties are powerful when combined with Query Embeds. You can filter notes and tasks by any property:

### Filter by Exact Value

```yaml
filters:
  - key: project
    operator: Equals
    value: "NeuroFlow"
```

### Filter by Existence

```yaml
filters:
  - key: due
    operator: Exists
```

### Filter by Partial Match

```yaml
filters:
  - key: status
    operator: Contains
    value: "progress"
```

See [Query Embeds Documentation](QUERY_EMBEDS.md) for full query syntax.

---

## Best Practices

### Consistent Naming

- Use lowercase keys: `project` not `Project`
- Use underscores or hyphens: `due_date` or `due-date`
- Be consistent across your vault

### Common Properties

| Property | Purpose | Example |
|----------|---------|---------|
| `project` | Group notes by project | `project: NeuroFlow` |
| `status` | Track note lifecycle | `status: draft` |
| `type` | Categorize notes | `type: meeting` |
| `due` | Set deadlines | `due: 2024-12-31` |
| `priority` | Prioritize items | `priority: high` |
| `tags` | Flexible categorization | `tags: [work, urgent]` |
| `created` | Track creation date | `created: 2024-01-15` |
| `author` | Track ownership | `author: Jane` |

### Property Inheritance

For notes linked to schedule blocks, consider using properties to categorize them:

```yaml
---
type: meeting
project: NeuroFlow
attendees:
  - Alice
  - Bob
---
```

This allows you to query all meetings, all notes for a project, or find notes by attendee.

---

## Troubleshooting

### Properties Not Showing

- Ensure the frontmatter is at the very top of the file (no blank lines before `---`)
- Check that the YAML is valid (proper indentation, no tabs)
- Re-index the vault if recently added

### Invalid YAML

Common YAML issues:
- Using tabs instead of spaces for indentation
- Missing quotes around values with special characters
- Incorrect list formatting

Valid:
```yaml
---
title: "My Note: A Story"
tags:
  - one
  - two
---
```

Invalid:
```yaml
---
title: My Note: A Story  # Colon needs quotes
tags:
	- one  # Tab instead of spaces
---
```

### Sync Issues

If property changes don't appear immediately:
1. Save the note
2. Close and reopen the note
3. If still not showing, try re-indexing the vault from Settings
