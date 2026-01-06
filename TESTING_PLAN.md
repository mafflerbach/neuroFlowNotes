# NeuroFlow Notes - Testing Plan

## 🎉 Completed Phases (Phases 1-4)

### Current Status
- **77 tests passing** in core_storage (100% pass rate)
- **Coverage**: ~58-62% of core_storage code
- **Target**: 65-70% coverage

### Completed Test Suites
1. ✅ **Tags Repository** (6 tests) - Replace, list, cascade delete
2. ✅ **Backlinks Repository** (6 tests) - Insert, update, bidirectional, cascade delete
3. ✅ **Notes Repository** (11 tests) - CRUD, upsert, rename, indexing
4. ✅ **Properties Repository** (9 tests) - Set, get, delete, batch queries
5. ✅ **Todos Repository** (8 tests) - Replace, query with filters, completion toggle
6. ✅ **Aliases Repository** (4 tests) - Case-insensitive lookup, cascade delete
7. ✅ **Schedule Repository** (13 tests) - RRULE expansion (7 patterns tested)
8. ✅ **Habits Repository** (12 tests) - CRUD, entries, tracker queries

### Additional Fixes Completed
- ✅ TypeScript errors fixed (0 errors)
- ✅ Clippy warnings fixed (0 warnings)
- ✅ Production unwrap() calls removed (13 fixed)
- ✅ Windows build fixed (icon.ico added)
- ✅ Linux build fixed (RGBA icons)
- ✅ CI/CD workflows restructured
- ✅ FolderTree.svelte null check added

---

## 🔜 Remaining Phases (Optional Enhancements)

### Phase 5: Dates Repository Tests
**Priority**: Medium | **Time**: 2-3 hours | **Tests**: 5

**Test File**: `crates/core_storage/tests/dates_repository.rs`

**Repository**: `crates/core_storage/src/repository/dates.rs`
- Method: `get_notes_for_date()` - Integration between schedules, journals, created dates

**Tests to Write**:
1. `test_get_notes_for_date_scheduled`
   - Create note with schedule block on 2024-01-15
   - Query notes for 2024-01-15
   - Verify note returned with source="scheduled"

2. `test_get_notes_for_date_journal`
   - Create note with property journal_date=2024-01-15
   - Query notes for 2024-01-15
   - Verify note returned with source="journal"

3. `test_get_notes_for_date_created`
   - Create note with created_date=2024-01-15
   - Query notes for 2024-01-15
   - Verify note returned with source="created"

4. `test_get_notes_for_date_priority_order`
   - Create 3 notes: scheduled, journal_date, created_date (all for same date)
   - Query notes for date
   - Verify order: scheduled first, journal second, created third

5. `test_get_notes_for_date_deduplication`
   - Create note with BOTH schedule block AND journal_date for same date
   - Query notes for date
   - Verify note appears only once (scheduled takes priority)

**Dependencies**:
- Requires schedule blocks and properties to be set up
- Tests integration between multiple tables

---

### Phase 6: Queries Repository Tests
**Priority**: Medium | **Time**: 4-5 hours | **Tests**: 8

**Test File**: `crates/core_storage/tests/queries_repository.rs`

**Repository**: `crates/core_storage/src/repository/queries.rs`
- Main method: `run_query()` - Query engine with property filters
- Supporting: `build_property_filter_sql()`, `get_matching_note_ids()`

**Tests to Write**:

#### Happy Path (6 tests)
1. `test_run_query_notes_only`
   - Create 3 notes with properties
   - Query with result_type=Notes, filter status=active
   - Verify only matching notes returned

2. `test_run_query_tasks_only`
   - Create notes with todos
   - Query with result_type=Tasks, include_completed=false
   - Verify only incomplete tasks returned

3. `test_run_query_both_notes_and_tasks`
   - Create notes with properties and todos
   - Query with result_type=Both
   - Verify both notes and tasks returned

4. `test_query_with_property_filter_equals`
   - Create notes with status property
   - Query with filter: key=status, operator=Equals, value=active
   - Verify only status=active notes returned

5. `test_query_with_property_filter_contains`
   - Create notes with tags property
   - Query with filter: key=tags, operator=Contains, value=work
   - Verify notes with "work" in tags returned

6. `test_query_with_multiple_filters_all_match`
   - Create notes with multiple properties
   - Query with 2 filters, match_mode=All (AND)
   - Verify only notes matching ALL filters returned

#### Error Paths (2 tests)
7. `test_query_with_multiple_filters_any_match`
   - Create notes with multiple properties
   - Query with 2 filters, match_mode=Any (OR)
   - Verify notes matching ANY filter returned

8. `test_query_empty_results`
   - Create notes
   - Query with filter that matches nothing
   - Verify empty results (not error)

**Dependencies**:
- `shared_types::{QueryRequest, PropertyFilter, PropertyOperator, FilterMatchMode, QueryResultType}`
- Complex SQL generation logic

---

### Phase 7: Integration Tests
**Priority**: Medium | **Time**: 3-4 hours | **Tests**: 8-10

**Test File**: `crates/core_storage/tests/integration_tests.rs`

**Purpose**: Test cross-repository operations and real-world workflows

**Tests to Write**:

#### Cross-Repository Operations (5 tests)
1. `test_note_deletion_cascades_all_relations`
   - Create note with: tags, properties, todos, backlinks, schedule blocks, aliases
   - Delete note
   - Verify ALL related data deleted from all tables

2. `test_full_note_indexing_workflow`
   - Create note content with frontmatter, todos, tags, links
   - Parse with core_index
   - Call index_note() with analysis
   - Verify all data stored correctly:
     - Note in notes table
     - Tags in tags table
     - Todos in todos table
     - Backlinks in backlinks table
     - FTS index updated

3. `test_property_sync_frontmatter_to_db`
   - Create note with frontmatter properties
   - Add DB-only property
   - Call replace_properties() with frontmatter
   - Verify frontmatter properties updated
   - Verify DB-only property preserved

4. `test_query_with_cross_table_filters`
   - Create notes with properties AND tags AND todos
   - Query with property filter + tag filter
   - Verify complex filtering works

5. `test_habit_tracker_with_schedule_blocks`
   - Create habit
   - Log entries
   - Create schedule blocks linked to habit notes
   - Query habit tracker
   - Verify entries and schedules both returned

#### Workflow Tests (3-5 tests)
6. `test_daily_note_workflow`
   - Create daily note
   - Add journal_date property
   - Add schedule blocks
   - Add todos
   - Query notes_for_date
   - Verify everything appears correctly

7. `test_rename_note_updates_backlinks`
   - Create note A linking to note B
   - Rename note B
   - Verify backlinks still reference correct note_id
   - (Note: May require core_domain logic)

8. `test_search_and_filter_integration`
   - Create 10 notes with varied properties, tags, content
   - Perform full-text search
   - Apply property filters to search results
   - Verify results correct

#### Performance Tests (optional, 2 tests)
9. `test_batch_operations_performance`
   - Create 100 notes
   - Batch insert properties for all
   - Verify performance is acceptable (<1 second)

10. `test_query_with_large_result_set`
    - Create 1000 todos across 100 notes
    - Query with limit=50
    - Verify limit applied correctly
    - Verify performance acceptable

**Dependencies**:
- Requires all repositories to be tested first
- Tests real-world workflows
- May require core_index and core_domain

---

### Phase 8: CI/CD Coverage Reporting
**Priority**: Low | **Time**: 2 hours

**Purpose**: Add coverage tracking (non-blocking)

#### Changes Needed

1. **Add tarpaulin to workspace dependencies**
   - File: `Cargo.toml` (workspace level)
   - Add `cargo-tarpaulin = "0.27"` to dev-dependencies

2. **Update CI workflow**
   - File: `.github/workflows/ci.yml`
   - Add coverage step after tests:
     ```yaml
     - name: Run tests with coverage
       run: cargo tarpaulin --workspace --out Xml --output-dir ./coverage
       
     - name: Upload coverage to Codecov
       uses: codecov/codecov-action@v3
       with:
         files: ./coverage/cobertura.xml
         fail_ci_if_error: false  # Non-blocking
     ```

3. **Add coverage badge to README**
   - File: `README.md`
   - Add: `[![codecov](https://codecov.io/gh/USERNAME/neuroFlowNotes/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/neuroFlowNotes)`

**Configuration**:
- Coverage tracking enabled but **non-blocking**
- PRs won't fail if coverage drops
- Coverage visible in PR comments for awareness
- Goal: Track progress toward 80% coverage

---

## 📊 Expected Final Coverage

### After Phase 5-7 Completion
- **Total tests**: ~93-95 tests
- **Estimated coverage**: 65-70% of core_storage
- **Time investment**: ~9-12 hours

### To Reach 80%+ Coverage (Future Work)
Still need:
- **core_domain tests** (Vault operations) - 15-20 tests
- **core_index tests** (Markdown parsing) - 10-15 tests
- **Command tests** (Tauri commands) - 20-25 tests

**Total additional**: ~45-60 tests needed beyond Phase 7

---

## 🚀 How to Resume Testing

### Quick Start
```bash
# Run existing tests to verify everything works
cargo test -p core_storage

# Start with Phase 5 (quickest win)
# Create: crates/core_storage/tests/dates_repository.rs
# Copy pattern from existing test files
```

### Test Patterns Established

**Basic Test Structure**:
```rust
//! Tests for the [repository_name] repository.

mod helpers;

use helpers::{insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_name() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Setup test data
    let note_id = insert_test_note(pool, "test.md", Some("Test")).await;
    
    // Execute operation
    let result = repo.some_method(note_id).await.unwrap();
    
    // Verify results
    assert_eq!(result.len(), 1);
}
```

**Happy Path + 2-3 Error Paths per Repository**

### Helper Functions Available
Located in: `crates/core_storage/tests/helpers/mod.rs`
- `setup_test_repo()` - Creates in-memory SQLite + VaultRepository
- `setup_test_db()` - Creates just the database pool
- `insert_test_note()` - Quick note creation
- `count_rows()` - Count rows in any table
- Additional helpers for properties, tags, etc.

---

## 📝 Notes

### Testing Strategy
- **Unit tests**: In-memory SQLite (`:memory:`) for speed
- **Integration tests**: File-based temp DB for realistic scenarios
- **Async tests**: Using `#[tokio::test]` macro
- **Pattern**: One test file per repository

### Code Quality Checklist
Before committing new tests:
- ✅ Run `cargo test -p core_storage` (all pass)
- ✅ Run `cargo clippy --workspace -- -D warnings` (0 warnings)
- ✅ Run `npm run check` (0 TypeScript errors)
- ✅ Format code: `cargo fmt`

### Commit Message Pattern
```
test: add [repository] repository tests

- Add X tests for [repository] repository
- Test [feature 1], [feature 2], [feature 3]
- Test cascade delete and error paths
- All tests passing (X/X)
```

---

## 🎯 Success Criteria

**Phase 5 Complete**: Dates repository fully tested (5 tests)  
**Phase 6 Complete**: Queries repository fully tested (8 tests)  
**Phase 7 Complete**: Integration tests cover main workflows (8-10 tests)  
**Phase 8 Complete**: Coverage reporting in CI (non-blocking)

**Overall Goal**: 80%+ test coverage with production-ready code quality

---

_Last Updated: January 6, 2026_  
_Current Test Count: 77 tests in core_storage_  
_Current Coverage: ~58-62% of core_storage_
