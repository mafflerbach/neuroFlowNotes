# NeuroFlow Notes

A modern, privacy-focused note-taking application with calendar integration, task management, and powerful query capabilities. Built with Tauri, Rust, and Svelte 5.

**Current Version:** v0.1.0

> **Status:** Active development. Core features are stable and functional. See [Roadmap](TODO.md) for planned features.

## What Makes NeuroFlow Different?

- **Calendar-First Design** - Unlike traditional note apps, NeuroFlow integrates scheduling directly into your workflow with daily, weekly, and monthly views
- **Schedule Blocks** - Time-based entries that link to notes, perfect for meetings, tasks, and time-blocking
- **Live Query Embeds** - Embed dynamic, filterable views of your notes and tasks directly in your documents
- **Obsidian-Compatible** - Full support for Obsidian's markdown syntax, wiki-links, frontmatter, and file structure
- **Plugin System** - Built-in LLM integration for summarization, habit tracking, and extensibility
- **Privacy-First** - All data stays local, optional LLM integration uses local models (LM Studio compatible)
- **Fast & Lightweight** - Rust backend with Svelte 5 frontend for blazing-fast performance

## Features

### Note Editor
- **Markdown-first editing** with CodeMirror 6
- **Live preview** - syntax hides on inactive lines, revealing formatted content
- **Wiki-style links** (`[[note]]`) with autocomplete and hover preview
- **Tags** support (`#tag`) with inline highlighting and autocomplete
- **Full-text search** across all notes (SQLite FTS5)
- **Code blocks** with syntax highlighting
- **Callouts** - Obsidian-style callout blocks (note, warning, info, etc.)
- **Custom embeds** - Embed notes, images, videos, and query results
- **Paste handling** - Auto-save pasted images to assets folder

### Properties & Metadata
- **YAML frontmatter** support for note metadata (fully Obsidian-compatible)
- **Two-way sync** - frontmatter is source of truth, synced to SQLite index
- **Properties panel** - view and edit note properties in the sidebar
- **Auto-parsing** - properties defined in YAML frontmatter are automatically indexed and displayed
- **Property types** - text, number, date, boolean, list with type-aware inputs
- **Optional conversion** - convert frontmatter to database properties (removes YAML, converts tags to inline #tags)
- **Bulk property management** - rename, merge, or delete properties across your entire vault
- **Usage tracking** - see which notes use each property value

### Tasks & Todos
- **Checkbox syntax** - `- [ ]` and `- [x]` for tasks
- **Task metadata** - priority, context, due dates inline
- **Query tasks** across all notes with filters
- **GTD contexts** - organize tasks by `@work`, `@home`, `@phone`, etc.

### Query Embeds
- **Live query results** embedded directly in notes
- **YAML-based syntax** in code blocks
- **Filter by properties** - equals, contains, exists, date operators, and more
- **Type-aware operators** - date comparisons (before/after), boolean, list matching
- **Multiple view types** - Table, List, or Kanban board
- **Kanban view** - group tasks by priority, context, or any property
- **Multi-tab queries** - multiple query tabs in a single embed
- **Sort and limit** results
- See [Query Embeds Documentation](docs/QUERY_EMBEDS.md) for details

### Calendar Integration
- **Three views**: Monthly, Weekly, and Daily
- **Schedule blocks** - create time-based entries on your calendar
- **Recurring appointments** with RRULE support (daily, weekly, monthly, weekdays, custom)
- **Link blocks to notes** - associate schedule blocks with existing notes
- **Drag & drop** to move schedule blocks between times/days

### File Management
- **Folder tree** with drag & drop file/folder organization
- **Inline renaming** - rename syncs H1 title, filename, and linked schedule blocks
- **Media support** - images, audio, and video with inline preview
- **Asset management** - paste images directly, auto-saved to assets folder
- **File type detection** - automatic MIME type detection for media files
- **Breadcrumb navigation** - visual path navigation in the editor

### Settings & Customization
- **Theme selection** - Light, Dark, or System auto-detection
- **Color schemes** - 19+ Catppuccin variants (Latte, Frappe, Macchiato, Mocha, etc.)
- **Block colors** - customize calendar block colors per-type
- **Properties Editor** - bulk manage property keys and values across your vault
- **Plugin settings** - configure built-in plugins from the settings panel
- **Vault settings** - customize daily note templates, default folders, and more

### Vault System
- **Local-first** - all data stored in plain Markdown files on your disk
- **SQLite index** for fast searching and metadata queries
- **Backlinks** tracking between notes
- **File watcher** - automatic re-indexing when files change externally
- **Semantic search** - vector embeddings for AI-powered note discovery (experimental)
- **No cloud required** - your data stays on your machine

### Plugin System
- **Built-in plugins** - extensible architecture with 5+ built-in plugins
  - **LLM File Summarizer** - Summarize notes using local LLMs (LM Studio compatible)
  - **LLM Daily Summarizer** - Create day summaries from schedule blocks and notes
  - **Habit Tracker** - Track daily habits with embeddable interactive tables
  - **Link Summarizer** - Summarize web links and save to notes
  - **Transcript Summarizer** - Process and summarize transcripts
- **Sidebar integration** - plugins can add custom panels to the sidebar
- **Calendar hooks** - plugins can add toolbar actions and context menus
- **Settings UI** - auto-generated settings interface from schema
- **Lifecycle hooks** - onEnable, onDisable, onSettingsChange
- **Backend access** - plugins can access notes, schedule blocks, and make HTTP requests

## Screenshots

*Coming soon*

## Installation

### From Releases (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/mafflerbach/neuroFlowNotes/releases) page:

- **macOS**: `.dmg` (Apple Silicon & Intel)
- **Windows**: `.msi` installer
- **Linux**: `.deb`, `.rpm`, or `.AppImage`

### Building from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- [Tauri CLI](https://tauri.app/start/prerequisites/)

#### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/mafflerbach/neuroFlowNotes.git
   cd neuroFlowNotes
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

4. Build for production:
   ```bash
   npm run tauri build
   ```

   The built application will be in `src-tauri/target/release/bundle/`.

## Quick Start

1. **Launch NeuroFlow Notes** and select or create a vault folder
2. **Create your first note** - Click the calendar or right-click in the folder tree
3. **Try these features:**
   - Link notes with `[[note name]]`
   - Add tags with `#tag`
   - Create tasks with `- [ ] Task description`
   - Add schedule blocks by clicking a time slot in the calendar
4. **Explore the sidebar** - Properties, backlinks, and plugin panels
5. **Customize** - Settings > Themes to change appearance

## Usage

### Getting Started

1. **Open a vault** - Select a folder to use as your vault (or create a new one)
2. **Create notes** - Right-click in the folder tree or use the calendar to create notes
3. **Link notes** - Use `[[note name]]` syntax to create links between notes
4. **Add properties** - Add YAML frontmatter or use the properties panel
5. **Schedule** - Click on the calendar to create schedule blocks for time-based planning
6. **Install plugins** - Settings > Plugins to enable built-in extensions

### Wiki Links

Create connections between notes using wiki-link syntax:

- `[[Note Name]]` - Link to another note
- `[[Note Name|Display Text]]` - Link with custom display text
- `[[Note Name#Section]]` - Link to a specific heading
- `![[image.png]]` - Embed an image
- `#tag` - Add a tag to a note

### Properties (Frontmatter)

Add metadata to notes using YAML frontmatter:

```markdown
---
project: NeuroFlow
status: active
priority: high
due: 2025-12-31
tags:
  - development
  - rust
aliases:
  - NeuroFlow App
  - NFN
---

# My Note Content
```

**Frontmatter Support:**
- **Full Obsidian compatibility** - Import your Obsidian vaults seamlessly
- **YAML parsing** - Robust parser handles all standard YAML types
- **Two-way sync** - Frontmatter is source of truth, synced to SQLite for fast queries
- **Property types** - Automatically detects strings, numbers, booleans, dates, and lists
- **Special fields** - `tags` and `aliases` are extracted and indexed separately
- **Visual editing** - Edit properties in the sidebar panel without touching YAML

**Optional Conversion**: When you complete typing a frontmatter block (closing `---`), NeuroFlow can optionally convert it to database-only properties, removing the YAML block and converting tags to inline `#tags`. This is useful if you prefer a cleaner note appearance while maintaining full property support.

**Bulk Operations** (in Settings):
- Rename property keys across all notes
- Rename property values across all notes
- Merge duplicate property keys
- Delete properties from all notes
- View usage statistics for each property

### Tasks

Create tasks with checkbox syntax:

```markdown
- [ ] Basic task
- [ ] Task with @context and !high priority
- [ ] Task with due date 📅 2025-12-25
- [x] Completed task
```

Query tasks across your vault with query embeds or the built-in task views.

### Schedule Blocks

Schedule blocks let you plan your day visually:

1. Click an empty time slot in the daily/weekly calendar view
2. Enter a title, time range, and optional recurrence pattern
3. Optionally link to an existing note
4. Click the block to open/create its associated note

Recurring patterns supported:
- Daily
- Weekly
- Every 2 weeks
- Monthly
- Weekdays (Mon-Fri)

## Documentation

- [Query Embeds](docs/QUERY_EMBEDS.md) - Live query syntax and examples
- [Properties](docs/PROPERTIES.md) - Working with note metadata and frontmatter
- [Settings](docs/SETTINGS.md) - Appearance, themes, and configuration
- [Agent Guide](AGENTS.md) - For AI coding agents working in this repository
- [Technical Specs](SPECS.md) - Detailed technical specification and architecture

## Architecture

```
neuroflow-notes/
├── src/                        # Svelte frontend
│   ├── lib/
│   │   ├── components/         # UI components
│   │   │   ├── calendar/       # Calendar view components
│   │   │   ├── query-builder/  # Query builder components
│   │   │   ├── folder-tree/    # File browser components
│   │   │   └── shared/         # Reusable components
│   │   ├── plugins/            # Plugin system
│   │   │   ├── builtin/        # Built-in plugins
│   │   │   │   ├── llm-file-summarizer/
│   │   │   │   ├── llm-daily-summarizer/
│   │   │   │   ├── habit-tracker/
│   │   │   │   ├── link-summarizer/
│   │   │   │   └── transcript-summarizer/
│   │   │   ├── registry.svelte.ts  # Plugin registry
│   │   │   ├── types.ts            # Plugin type definitions
│   │   │   └── api.ts              # Backend hooks for plugins
│   │   ├── stores/             # Svelte 5 stores (state management)
│   │   ├── services/           # API layer
│   │   │   ├── api/            # Domain-specific API modules
│   │   │   └── client.ts       # API wrapper with error handling
│   │   ├── editor/             # CodeMirror 6 extensions
│   │   │   ├── editorConfig.ts     # Main editor configuration
│   │   │   ├── linkHandler.ts      # Wiki-link support
│   │   │   ├── embedExtension.ts   # Query/note embeds
│   │   │   ├── calloutExtension.ts # Callout blocks
│   │   │   └── frontmatterConversion.ts # YAML conversion
│   │   ├── types/              # TypeScript type definitions
│   │   │   └── bindings/       # Auto-generated from Rust (ts-rs)
│   │   └── utils/              # Helper functions
│   └── App.svelte              # Main application
├── src-tauri/                  # Tauri/Rust backend
│   └── src/
│       ├── commands/           # IPC command handlers (modular)
│       │   ├── vault.rs        # Vault open/close/info
│       │   ├── notes.rs        # Note CRUD
│       │   ├── properties.rs   # Property management
│       │   ├── schedule.rs     # Calendar/schedule blocks
│       │   ├── queries.rs      # Query execution
│       │   ├── plugins.rs      # Plugin config I/O
│       │   └── mod.rs          # Command exports
│       ├── main.rs             # Tauri app entry point
│       └── state.rs            # Application state
└── crates/                     # Rust workspace crates
    ├── shared_types/           # DTOs with ts-rs bindings
    │   └── types/              # Modular type definitions
    ├── core_fs/                # File system operations
    ├── core_index/             # Markdown parsing & indexing
    │   ├── frontmatter.rs      # YAML frontmatter parser
    │   └── markdown.rs         # Markdown AST parser
    ├── core_storage/           # SQLite database layer
    │   ├── schema.rs           # Database schema
    │   └── repository/         # Query repositories
    ├── core_domain/            # Business logic (vault operations)
    │   ├── vault.rs            # Vault management
    │   ├── watcher.rs          # File system watcher
    │   └── templates.rs        # Template system
    └── core_embedding/         # Vector embeddings (experimental)
        ├── client.rs           # Qdrant client
        └── queue.rs            # Embedding queue
```

## Tech Stack

- **Frontend**: Svelte 5 (with runes), TypeScript, CodeMirror 6
- **Backend**: Rust, Tauri 2
- **Database**: SQLite (via sqlx) with FTS5 full-text search
- **Embeddings**: Qdrant vector database (optional, for semantic search)
- **Styling**: CSS with Catppuccin color schemes
- **Markdown**: Custom parser with frontmatter support (serde_yaml)

## Development

### Running Tests

```bash
# Rust tests
cargo test

# TypeScript type checking
npm run check
```

### Project Structure

The project uses a Rust workspace with multiple crates for separation of concerns:

- `shared_types` - Data transfer objects with TypeScript generation (ts-rs)
- `core_fs` - Platform-agnostic file operations
- `core_index` - Markdown parsing, frontmatter extraction, tag/link extraction
- `core_storage` - Database schema and queries (SQLite via sqlx)
- `core_domain` - High-level vault operations, file watcher, template system
- `core_embedding` - Vector embeddings and semantic search (Qdrant client)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [Svelte](https://svelte.dev/) - Reactive UI framework
- [CodeMirror](https://codemirror.net/) - Powerful editor component
- [Catppuccin](https://catppuccin.com/) - Beautiful color themes
- [Lucide](https://lucide.dev/) - Icon set
