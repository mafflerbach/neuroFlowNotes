# NeuroFlow Notes

A modern, privacy-focused note-taking application with calendar integration, task management, and powerful query capabilities. Built with Tauri, Rust, and Svelte 5.

## Features

### Note Editor
- **Markdown-first editing** with CodeMirror 6
- **Live preview** - syntax hides on inactive lines, revealing formatted content
- **Wiki-style links** (`[[note]]`) with autocomplete
- **Tags** support (`#tag`) with inline highlighting
- **Full-text search** across all notes
- **Code blocks** with syntax highlighting

### Properties (Frontmatter)
- **YAML frontmatter** support for note metadata
- **Visual property editor** - add/edit properties without touching YAML
- **Property types** - text, number, date, checkbox, list
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
- **Filter by properties** - equals, contains, exists, and more
- **Table or list view** with customizable columns
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

### Settings & Customization
- **Theme selection** - Light, Dark, or System auto-detection
- **Color schemes** - Catppuccin Latte, Frappe, Macchiato, Mocha
- **Block colors** - customize calendar block colors per-type
- **Properties Editor** - bulk manage property keys and values across your vault

### Vault System
- **Local-first** - all data stored in plain Markdown files on your disk
- **SQLite index** for fast searching and metadata queries
- **Backlinks** tracking between notes
- **No cloud required** - your data stays on your machine

## Screenshots

*Coming soon*

## Installation

### From Releases (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/your-username/neuroflow-notes/releases) page:

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
   git clone https://github.com/your-username/neuroflow-notes.git
   cd neuroflow-notes
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

## Usage

### Getting Started

1. **Open a vault** - Select a folder to use as your vault (or create a new one)
2. **Create notes** - Right-click in the folder tree or use the calendar to create notes
3. **Link notes** - Use `[[note name]]` syntax to create links between notes
4. **Add properties** - Use the properties panel or YAML frontmatter
5. **Schedule** - Click on the calendar to create schedule blocks for time-based planning

### Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Save note | `Cmd/Ctrl + S` |
| Search | `Cmd/Ctrl + K` |
| Bold | `Cmd/Ctrl + B` |
| Italic | `Cmd/Ctrl + I` |
| New note | `Cmd/Ctrl + N` |

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
due: 2024-12-31
tags:
  - development
  - rust
---

# My Note Content
```

Properties appear in the sidebar panel and can be edited visually.

### Tasks

Create tasks with checkbox syntax:

```markdown
- [ ] Basic task
- [ ] Task with @context and !high priority
- [ ] Task with due date 📅 2024-12-25
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
- [Properties](docs/PROPERTIES.md) - Working with note metadata
- [Settings](docs/SETTINGS.md) - Appearance, themes, and configuration

## Architecture

```
neuroflow-notes/
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   ├── stores/         # Svelte stores (state management)
│   │   ├── services/       # API & backend communication
│   │   ├── editor/         # CodeMirror extensions
│   │   └── utils/          # Helper functions
│   └── App.svelte          # Main application
├── src-tauri/              # Tauri/Rust backend
│   └── src/
│       └── commands.rs     # IPC command handlers
└── crates/                 # Rust workspace crates
    ├── shared_types/       # DTOs shared between frontend/backend
    ├── core_fs/            # File system operations
    ├── core_index/         # Markdown parsing & indexing
    ├── core_storage/       # SQLite database layer
    └── core_domain/        # Business logic (vault operations)
```

## Tech Stack

- **Frontend**: Svelte 5, TypeScript, CodeMirror 6
- **Backend**: Rust, Tauri 2
- **Database**: SQLite (via sqlx)
- **Styling**: CSS with Catppuccin color schemes

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

- `shared_types` - Data transfer objects with TypeScript generation
- `core_fs` - Platform-agnostic file operations
- `core_index` - Markdown parsing, tag/link extraction
- `core_storage` - Database schema and queries
- `core_domain` - High-level vault operations

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
