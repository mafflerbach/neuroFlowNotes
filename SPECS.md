# 🧠 **NeuroFlow Notes – Technical Specification (v1.2)**

> **Implementation Status:** Milestone 1 & 2 Complete, UI Refactor Complete
>
> - Backend: Vault indexing, file watching, SQLite storage ✅
> - Frontend: Svelte 5 with CodeMirror 6, Calendar-centric UI ✅
> - Pending: Properties API, Schedule Blocks API, Data Wiring

## 1. Overview

**NeuroFlow Notes** is a local-first second brain application designed as a hybrid between Obsidian, Jibun Techo planners, and modern productivity dashboards.

### 1.1 Core Design Principles

* Notes stored as **plain Markdown files** in a normal folder structure.
* **No proprietary format** for text.
* All **metadata, links, tags, tasks, and scheduling** stored in **SQLite**.
* **Filesystem is source of truth** – SQLite is a cache + metadata layer.
* Full support for linking (`[[note name]]`) and tagging (`#tag`).
* Integrated **Daily View**, **Weekly Timeline View**, and **Scheduled Blocks**.
* Clicking a scheduled block opens:

  * The linked note in a main editor pane.
  * A **task inspector** pane showing todos extracted from that note.
* Clear **separation of concerns** in architecture.
* Frontend plugins in **TypeScript/JavaScript**, not Rust.

---

## 2. Technology Stack

### 2.1 Core Application

* **Tauri** (desktop application shell)
* **Rust** for backend logic
* **SQLite** (with FTS5) for metadata + search
* **Tauri commands** as IPC boundary between backend & UI

### 2.2 Frontend

* Framework: **Svelte + TypeScript** (React also possible)
* Markdown editing: **CodeMirror 6**
* Plugin system: **JavaScript/TypeScript plugins** loaded in frontend
* UI built as a modular component system:

  * WeeklyView
  * DailyView
  * NoteEditor
  * TodoPanel
  * CalendarPicker
  * Sidebar (folders, tags, views)

---

## 3. File Storage Model (Vault)

A vault is any directory:

```
vault/
  daily/
  projects/
  areas/
  templates/
  .neuroflow/
```

Notes are **Markdown files**:

```
vault/
  daily/2025-12-06.md
  projects/neural-orchestrator.md
  areas/health.md
```

Rules:

* Markdown files contain *only human-editable content*.
* No required YAML frontmatter.
* Inline tags (`#tag`) and links (`[[note]]`) fully supported.

---

## 4. Metadata & SQLite Schema

SQLite DB located at:

```
vault/.neuroflow/neuroflow.db
```

### 4.1 Tables

```sql
CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  title TEXT,
  created_at TEXT,
  updated_at TEXT,
  hash TEXT,
  pinned INTEGER DEFAULT 0
);

CREATE TABLE properties (
  id INTEGER PRIMARY KEY,
  note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  key TEXT NOT NULL,
  value TEXT,
  type TEXT,
  sort_order INTEGER
);

CREATE TABLE tags (
  id INTEGER PRIMARY KEY,
  note_id INTEGER NOT NULL REFERENCES notes(id),
  tag TEXT NOT NULL
);

CREATE TABLE backlinks (
  id INTEGER PRIMARY KEY,
  from_note_id INTEGER NOT NULL REFERENCES notes(id),
  to_note_id INTEGER NOT NULL REFERENCES notes(id)
);

CREATE TABLE schedule_blocks (
  id INTEGER PRIMARY KEY,
  note_id INTEGER NOT NULL REFERENCES notes(id),
  date TEXT NOT NULL,
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  label TEXT,
  color TEXT,
  context TEXT
);

CREATE TABLE todos (
  id INTEGER PRIMARY KEY,
  note_id INTEGER NOT NULL REFERENCES notes(id),
  line_number INTEGER,
  description TEXT NOT NULL,
  completed INTEGER NOT NULL DEFAULT 0,
  heading_path TEXT,
  created_at TEXT,
  completed_at TEXT
);

CREATE VIRTUAL TABLE notes_fts USING fts5(
  note_id UNINDEXED,
  content
);
```

---

## 5. Architecture & Separation of Concerns

### 5.1 Rust Workspace Structure

```
core_fs/          -- reading & writing .md files
core_index/       -- parsing markdown & updating DB
core_storage/     -- SQLite repository layer (sqlx)
core_domain/      -- business rules (daily view, weekly view, scheduling)
app_tauri/        -- Tauri commands exposing domain to UI
shared_types/     -- DTO structs for Rust <-> TypeScript
```

### 5.2 Frontend Structure

```
src/components/
src/services/     -- wrappers for Tauri commands
src/stores/       -- Svelte stores / global state
src/pluginHost/   -- JS plugin loader + sandbox API
src/views/
```

### 5.3 Clean Code Rules

* UI never touches the filesystem or SQLite.
* UI talks **only** to a TypeScript service layer.
* Service layer calls **Tauri commands**.
* Tauri commands call domain methods.
* Domain interacts with filesystem + storage through abstractions.
* Indexing layer is isolated from domain logic and UI concerns.
* Plugins talk to a **sandboxed frontend API** only.

---

## 6. Markdown Parsing

### 6.1 Crate Choice

* **Primary parser:** `pulldown-cmark`
  * Fast, well-maintained, streaming, no heap-heavy AST by default.
  * Walk the event stream and build only what's needed.
* **Not using:** `comrak` – heavier, opinionated (CommonMark + GFM), unnecessary since all visual rendering happens in CodeMirror/frontend.

### 6.2 Parsing Strategy

**Hybrid approach:**

1. Run `pulldown-cmark` over the whole file to:
   * Find headings (for `heading_path`)
   * Find list items
   * Detect markdown tasks `- [ ]` and `- [x]`

2. Run a secondary, cheap string/regex pass for:
   * `[[wikilinks]]`
   * `#tags`

This avoids overcomplicating the markdown parser for patterns that are essentially lexical.

### 6.3 NoteAnalysis Struct

```rust
struct NoteAnalysis {
    title: Option<String>,
    headings: Vec<Heading>,
    todos: Vec<ParsedTodo>,
    tags: Vec<String>,
    links: Vec<String>, // target strings, resolved later
}
```

Lives in `core_index::markdown`.

### 6.4 Separation of Concerns

* `core_fs` → reads raw string from disk.
* `core_index` → turns string into `NoteAnalysis`.
* `core_storage` → persists `NoteAnalysis` into DB.

---

## 7. File Watching & Sync Strategy

### 7.1 Crate Choice

Use the `notify` crate for filesystem watching.

### 7.2 Initialization

On vault open:

1. Do a full scan of all `.md` files.
2. Build initial DB index.
3. Start a `notify` watcher on the root vault folder, recursively.

### 7.3 Event Handling

Handle `Create`, `Write`, `Remove`, `Rename` for `*.md` files:

| Event | Action |
|-------|--------|
| Create / Write | Add `IndexJob::Reindex(path)` to queue |
| Remove | Add `IndexJob::Remove(path)` to queue |

Use a **debouncer**: if multiple events come quickly for the same file, only reindex once.

### 7.4 Sync Logic Per File

1. Read file metadata (`mtime`) + optionally hash (e.g., `xxhash`).
2. Compare to `notes.hash` and `notes.updated_at`.
3. If changed → re-parse and upsert metadata in a DB transaction.
4. If file gone → delete row + cascade.

### 7.5 Internal Edits (from UI)

When changes originate from inside the app (editing Markdown via UI):

1. Write to file via `core_fs`.
2. Trigger the same reindex path as if an external change happened.

**One flow for "this file changed"** – no divergent logic.

---

## 8. Todo Sync (Panel → Markdown)

### 8.1 Flow When User Checks a Box

1. Fetch current content from disk (`core_fs::read`).
2. Re-parse with `pulldown-cmark` + `NoteAnalysis`.
3. Rebuild the markdown string with the task toggled.
4. Write back entire file.
5. Re-index and store updated todos in DB.

### 8.2 Implementation Note

Keep `NoteAnalysis` in memory after parse to avoid re-parsing twice:

```rust
let analysis = parse(&content);
let new_content = rebuild_with_toggle(&content, &analysis, todo_line);
```

Full file rewrite is simple and correct. Line-based patching is fragile (what if user added lines above?). Overhead of rewriting a `.md` file is negligible.

---

## 9. User Interface Model

### 9.1 Weekly View (Outlook/Jibun Hybrid)

* 7 columns (Mon–Sun)
* Customizable vertical timeline:

  * configurable start hour
  * configurable end hour
  * 15/30/60 min granularity
* Schedule blocks displayed as cards
* Clicking a block:

  * Opens the linked note in center pane
  * Opens the todo panel in right pane

### 9.2 Daily View

* Single timeline
* All blocks for that date
* Actions:

  * Quick-add new block
  * Create/open daily note
  * Show metadata for the day (optional)

### 9.3 Note Editor

* Uses CodeMirror 6
* Supports:

  * Live edit
  * Inline preview
  * `[[links]]`
  * `#tags`
  * Markdown tasks `- [ ]`

### 9.4 Todo Panel

* Extracted from parsed markdown
* Shows open tasks
* Clicking a checkbox updates the markdown + DB
* Groups tasks by headings

### 9.5 Sidebar

* Folder tree
* Tag browser
* Saved views
* Calendar picker

### 9.6 Schedule Block Creation (MVP UX)

**Click on empty slot in timeline:**

1. Open a small popover/modal with:
   * Start/end times (pre-filled from slot or minimal duration)
   * Note link:
     * If a note is currently open in editor → preselect that note
     * Else: a "quick search" input to select a note by title/path
   * Optional: label, color, context

**Block resizing / drag-and-drop:** Nice to have (v2), not MVP.

**Domain rule:** Block creation lives in `core_domain::schedule`, not in UI.

---

## 10. Daily Notes

### 10.1 When to Create

**On first access**, not on app open.

Triggers:
* Menu item / button: "Open today's note"
* Keyboard shortcut: e.g., `Ctrl/Cmd + D`

### 10.2 Creation Flow

When triggered:

1. If `daily/YYYY-MM-DD.md` exists → open it.
2. Else:
   * Create from template.
   * Write file.
   * Trigger index.
   * Open in editor.

Optional config flag (future): "auto-create today's note on app start".

### 10.3 Template Syntax

Simple variable substitution: `{{date}}`, `{{weekday}}`, `{{week}}`, `{{year}}`, etc.

Example template (`templates/daily.md`):

```markdown
# {{date}} – {{weekday}}

## Plan
- [ ] Top 1
- [ ] Top 2

## Log

## Reflection
- What went well?
- What was hard?
```

Implementation:

```rust
fn render_template(input: &str, ctx: &TemplateContext) -> String
```

Lives in `core_domain::templates`. UI just calls `create_or_open_daily_note(date)`.

---

## 11. Frontend State Sync

### 11.1 Event-Driven Updates

Use Tauri's event system, not polling.

### 11.2 Backend → Frontend Events

| Event | Payload | Frontend Action |
|-------|---------|-----------------|
| `notes:updated` | `{ noteIds: number[] }` | Refresh affected note views |
| `notes:deleted` | `{ noteIds: number[] }` | Remove from UI, close if open |
| `index:complete` | `{}` | Refresh sidebar, update counts |

### 11.3 Frontend Subscription

```ts
import { listen } from "@tauri-apps/api/event";

listen("notes:updated", (event) => {
  noteStore.invalidate(event.payload.noteIds);
});
```

---

## 12. Database Architecture

### 12.1 Two Separate Databases

**Vault Database** (`vault/.neuroflow/neuroflow.db`)
* Purpose: Everything that can be recreated from Markdown files or is logically tied to that vault.
* Contains: notes, tags, backlinks, todos, schedule_blocks, properties, notes_fts
* Can be deleted and rebuilt from markdown at any time.

**App Database** (platform-specific app data directory)
* Purpose: Everything about the app, not the vault.
* Contains: recent vaults, window state, global preferences, plugin settings
* Never depends on actual Markdown content.
* If you move/rename a vault, worst case is "recent vaults" entry breaks.

### 12.2 Rationale

Clear separation ensures:
* Vault portability (copy folder = copy everything)
* Safe rebuilds (delete vault DB, re-index, no data loss)
* App settings persist across vault switches

---

## 13. Plugin System (JS/TS)

Plugins live in:

```
vault/.neuroflow/plugins/<plugin-id>/
  manifest.json
  main.js
```

### 13.1 `manifest.json`

```json
{
  "id": "daily-review",
  "name": "Daily Review",
  "version": "0.1.0",
  "entry": "main.js",
  "permissions": ["notes:read", "todos:read", "ui:panel"]
}
```

### 13.2 Plugin API

Plugins receive a restricted `NeuroflowAPI` object:

```ts
interface NeuroflowAPI {
  notes: { list, get, getByPath, open };
  todos: { listForNote, update };
  schedule: { listForWeek, createBlock, updateBlock };
  ui: { registerCommand, registerPanel, showNotification };
  date: { today, startOfWeek };
}
```

### 13.3 Example Plugin

```ts
export function activate(api) {
  api.ui.registerCommand({
    id: "daily-review",
    title: "Run Daily Review",
    run: async () => {
      const today = api.date.today();
      const note = await api.notes.getByPath(`daily/${today}.md`);
      const todos = await api.todos.listForNote(note.id, { completed: false });
      api.ui.openPanel("daily-review", DailyReviewPanel, { note, todos });
    }
  });
}

export function deactivate() {}
```

### 13.4 Plugin Sandboxing

**Reality:** In a Tauri webview, all JS runs in the same process/context. True Chrome-level origin isolation is not achievable.

**v1 Approach:** API sandboxing + convention (same as VS Code & Obsidian).

**Enforcement:**

1. Plugins are loaded only from `.neuroflow/plugins/`.
2. Plugin code never gets `window.__TAURI__` or raw `invoke`.
3. Only the `NeuroflowAPI` object is passed to `activate(api)`.
4. No dangerous APIs exposed (no raw file write, etc.) unless explicitly intended.

**Implementation in `pluginHost`:**

1. Scan plugin metadata.
2. `import()` each plugin's `main.js` in a controlled scope.
3. Call `activate(api)`.
4. Do not re-export raw Tauri bindings into that scope.

**Plugins are "trusted code"** – users install at their own risk (documented clearly).

---

## 14. Customization Config

Stored at:

```
vault/.neuroflow/config.json
```

Example:

```json
{
  "day_start_hour": 6,
  "day_end_hour": 22,
  "slot_interval_minutes": 30,
  "default_daily_note_folder": "daily",
  "templates": {
    "daily": "templates/daily.md",
    "weekly": "templates/weekly.md"
  }
}
```

---

## 15. MVP Milestones

### Completed

1. ✅ Vault open + indexing (Rust)
2. ✅ NoteEditor + folder tree (frontend)
3. ✅ SQLite metadata extraction:
   * tags
   * links
   * todos
4. ✅ Calendar-centric UI with State A/B/C
5. ✅ Weekly View component (Outlook-style)
6. ✅ Daily View component (vertical timeline)
7. ✅ Monthly View component (dots + list)

### In Progress

8. 🔄 Properties API (CRUD for key-value metadata)
9. 🔄 Schedule Blocks API (CRUD + queries by date)
10. 🔄 Notes by Date query (scheduled > journal > created)
11. 🔄 Wire calendar components to real backend data

### Pending

12. ⏳ Daily notes with templates
13. ⏳ Link resolution (click [[wikilink]] → open note)
14. ⏳ Schedule block creation UI
15. ⏳ JS plugin system (loading + basic API)

