# Contributing to NeuroFlow Notes

Thank you for your interest in contributing to NeuroFlow Notes! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- **Node.js** 18+ ([nodejs.org](https://nodejs.org/))
- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Tauri CLI** 2.x ([tauri.app](https://tauri.app/start/prerequisites/))

### Getting Started

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/neuroFlowNotes.git
   cd neuroFlowNotes
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Start development server:**
   ```bash
   npm run tauri dev
   ```

4. **Run tests:**
   ```bash
   # Rust tests
   cargo test

   # TypeScript type checking
   npm run check
   ```

## Project Structure

### Frontend (TypeScript/Svelte)

```
src/lib/
├── components/           # Svelte components
│   ├── calendar/         # Calendar view components
│   ├── query-builder/    # Query builder (split into subcomponents)
│   ├── shared/           # Reusable components
│   └── *.svelte          # Feature components
├── editor/               # CodeMirror extensions
├── services/             # API layer
│   ├── api/              # Domain-specific API modules
│   └── client.ts         # API wrapper with error handling
├── stores/               # Svelte state management
├── types/                # TypeScript type definitions
└── utils/                # Helper functions
```

### Backend (Rust)

```
src-tauri/src/
├── commands/             # Tauri command handlers (modular)
│   ├── mod.rs            # Re-exports and error types
│   ├── vault.rs          # Vault operations
│   ├── notes.rs          # Note CRUD
│   ├── properties.rs     # Property management
│   └── ...               # Other modules
├── state.rs              # Application state
└── main.rs               # Entry point

crates/
├── shared_types/         # DTOs (with ts-rs TypeScript generation)
├── core_fs/              # File system abstraction
├── core_index/           # Markdown parsing & indexing
├── core_storage/         # SQLite database layer
└── core_domain/          # Business logic
```

## Code Style

### TypeScript/Svelte

- Use TypeScript strict mode
- Prefer functional patterns
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Follow existing naming conventions (camelCase for variables, PascalCase for components)

### Rust

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Use `#[instrument]` for tracing in command handlers
- Document public APIs with doc comments
- Keep command handlers thin - delegate to domain layer

## Making Changes

### Before Starting

1. Check existing issues and PRs
2. For significant changes, open an issue first to discuss the approach
3. Keep changes focused - one feature/fix per PR

### Commit Messages

Use clear, descriptive commit messages:

```
Add date filter operators to query builder

- Add DateOn, DateBefore, DateAfter, DateOnOrBefore, DateOnOrAfter operators
- Update QueryBuilder UI with date picker for date operators
- Add SQL date comparison logic in queries.rs
```

### Pull Request Process

1. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes** with clear commits

3. **Test your changes:**
   ```bash
   cargo test
   npm run check
   npm run tauri dev  # Manual testing
   ```

4. **Push and open a PR** against `main`

5. **Describe your changes** in the PR description:
   - What does this change?
   - Why is it needed?
   - How was it tested?

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p core_storage

# Run with logging
RUST_LOG=debug cargo test
```

### TypeScript Checks

```bash
# Type checking
npm run check

# Build check
npm run build
```

### Manual Testing

When testing UI changes:
1. Test with different themes (light/dark, all color schemes)
2. Test with sample vaults containing various note types
3. Test edge cases (empty states, long content, special characters)

## Need Help?

- Open an issue for bugs or feature requests
- Ask questions in discussions
- Check existing documentation in `/docs`

Thank you for contributing!
