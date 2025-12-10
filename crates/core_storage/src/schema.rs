//! Database schema initialization.

use sqlx::SqlitePool;
use tracing::{info, debug};

/// Initialize the database schema.
pub async fn init_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Initializing database schema");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            title TEXT,
            created_at TEXT,
            updated_at TEXT,
            hash TEXT,
            pinned INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_path ON notes(path);
        CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS properties (
            id INTEGER PRIMARY KEY,
            note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT,
            type TEXT,
            sort_order INTEGER,
            UNIQUE(note_id, key)
        );

        CREATE INDEX IF NOT EXISTS idx_properties_note_id ON properties(note_id);
        CREATE INDEX IF NOT EXISTS idx_properties_key ON properties(key);
        CREATE INDEX IF NOT EXISTS idx_properties_value ON properties(value);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            tag TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tags_note_id ON tags(note_id);
        CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS backlinks (
            id INTEGER PRIMARY KEY,
            from_note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            to_note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_backlinks_from ON backlinks(from_note_id);
        CREATE INDEX IF NOT EXISTS idx_backlinks_to ON backlinks(to_note_id);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schedule_blocks (
            id INTEGER PRIMARY KEY,
            note_id INTEGER REFERENCES notes(id) ON DELETE CASCADE,
            date TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            label TEXT,
            color TEXT,
            context TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_schedule_blocks_note_id ON schedule_blocks(note_id);
        CREATE INDEX IF NOT EXISTS idx_schedule_blocks_date ON schedule_blocks(date);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            line_number INTEGER,
            description TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            heading_path TEXT,
            created_at TEXT,
            completed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_todos_note_id ON todos(note_id);
        CREATE INDEX IF NOT EXISTS idx_todos_completed ON todos(completed);
        "#,
    )
    .execute(pool)
    .await?;

    // Create FTS table for full-text search
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            content,
            content='',
            contentless_delete=1
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Migration: Fix schedule_blocks table if note_id has NOT NULL constraint
    // SQLite doesn't support ALTER TABLE to change constraints, so we need to recreate the table
    migrate_schedule_blocks(pool).await?;

    // Migration: Ensure properties table has UNIQUE(note_id, key) constraint
    migrate_properties(pool).await?;

    // Migration: Add created_date column for local date storage
    migrate_created_date(pool).await?;

    // Migration: Add rrule column for recurring schedule blocks
    migrate_schedule_blocks_rrule(pool).await?;

    info!("Database schema initialized");
    Ok(())
}

/// Migrate notes table to add created_date column for local date storage.
/// This avoids timezone issues with UTC timestamps.
async fn migrate_created_date(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if created_date column exists
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT cid, name, type, `notnull`, dflt_value, pk FROM pragma_table_info('notes')"
    )
    .fetch_all(pool)
    .await?;

    let has_created_date = columns.iter().any(|(_, name, _, _, _, _)| name == "created_date");

    if !has_created_date {
        info!("Migrating notes table: adding created_date column");

        // Add the column
        sqlx::query("ALTER TABLE notes ADD COLUMN created_date TEXT")
            .execute(pool)
            .await?;

        // Backfill existing notes with date extracted from created_at (UTC, but better than nothing)
        sqlx::query("UPDATE notes SET created_date = date(created_at) WHERE created_date IS NULL AND created_at IS NOT NULL")
            .execute(pool)
            .await?;

        // Create index for faster queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_created_date ON notes(created_date)")
            .execute(pool)
            .await?;

        info!("notes table migration complete: added created_date column");
    } else {
        debug!("notes.created_date column already exists");
    }

    Ok(())
}

/// Migrate schedule_blocks table to allow NULL note_id.
/// This is needed because older versions had NOT NULL constraint.
async fn migrate_schedule_blocks(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if the column allows NULL by looking at table_info pragma
    let column_info: Vec<(i64, String, String, i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT cid, name, type, `notnull`, dflt_value, pk FROM pragma_table_info('schedule_blocks') WHERE name = 'note_id'"
    )
    .fetch_all(pool)
    .await?;

    if let Some((_cid, _name, _type, notnull, _dflt_value, _pk)) = column_info.first() {
        if *notnull == 1 {
            info!("Migrating schedule_blocks table: removing NOT NULL constraint from note_id");

            // SQLite doesn't support ALTER TABLE to change constraints
            // We need to recreate the table

            // First, drop temp table if it exists from a failed migration
            sqlx::query("DROP TABLE IF EXISTS schedule_blocks_new")
                .execute(pool)
                .await?;

            // Create new table with correct schema
            sqlx::query(
                r#"
                CREATE TABLE schedule_blocks_new (
                    id INTEGER PRIMARY KEY,
                    note_id INTEGER REFERENCES notes(id) ON DELETE CASCADE,
                    date TEXT NOT NULL,
                    start_time TEXT NOT NULL,
                    end_time TEXT NOT NULL,
                    label TEXT,
                    color TEXT,
                    context TEXT
                )
                "#,
            )
            .execute(pool)
            .await?;

            // Copy existing data
            sqlx::query(
                "INSERT INTO schedule_blocks_new SELECT * FROM schedule_blocks"
            )
            .execute(pool)
            .await?;

            // Drop old table
            sqlx::query("DROP TABLE schedule_blocks")
                .execute(pool)
                .await?;

            // Rename new table
            sqlx::query("ALTER TABLE schedule_blocks_new RENAME TO schedule_blocks")
                .execute(pool)
                .await?;

            // Recreate indexes
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_schedule_blocks_note_id ON schedule_blocks(note_id)")
                .execute(pool)
                .await?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_schedule_blocks_date ON schedule_blocks(date)")
                .execute(pool)
                .await?;

            info!("schedule_blocks table migration complete");
        } else {
            debug!("schedule_blocks.note_id already allows NULL, no migration needed");
        }
    }

    Ok(())
}

/// Migrate properties table to ensure UNIQUE(note_id, key) constraint exists.
async fn migrate_properties(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if the UNIQUE constraint exists by looking at index_list pragma
    let indexes: Vec<(i64, String, i64, String, i64)> = sqlx::query_as(
        "SELECT seq, name, `unique`, origin, partial FROM pragma_index_list('properties')"
    )
    .fetch_all(pool)
    .await?;

    // Check if there's a unique index on (note_id, key)
    let has_unique_constraint = indexes.iter().any(|(_, name, unique, _, _)| {
        *unique == 1 && name.contains("note_id") && name.contains("key")
    });

    // Also check for sqlite_autoindex which is created for UNIQUE constraints in table definition
    let has_autoindex = indexes.iter().any(|(_, name, unique, _, _)| {
        *unique == 1 && name.starts_with("sqlite_autoindex_properties_")
    });

    if !has_unique_constraint && !has_autoindex {
        info!("Migrating properties table: adding UNIQUE(note_id, key) constraint");

        // Drop temp table if it exists from a failed migration
        sqlx::query("DROP TABLE IF EXISTS properties_new")
            .execute(pool)
            .await?;

        // Create new table with correct schema
        sqlx::query(
            r#"
            CREATE TABLE properties_new (
                id INTEGER PRIMARY KEY,
                note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                value TEXT,
                type TEXT,
                sort_order INTEGER,
                UNIQUE(note_id, key)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Copy existing data (deduplicate if necessary - keep latest)
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO properties_new (id, note_id, key, value, type, sort_order)
            SELECT id, note_id, key, value, type, sort_order FROM properties
            "#
        )
        .execute(pool)
        .await?;

        // Drop old table
        sqlx::query("DROP TABLE properties")
            .execute(pool)
            .await?;

        // Rename new table
        sqlx::query("ALTER TABLE properties_new RENAME TO properties")
            .execute(pool)
            .await?;

        // Recreate indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_properties_note_id ON properties(note_id)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_properties_key ON properties(key)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_properties_value ON properties(value)")
            .execute(pool)
            .await?;

        info!("properties table migration complete");
    } else {
        debug!("properties table already has UNIQUE(note_id, key) constraint");
    }

    Ok(())
}

/// Migrate schedule_blocks table to add rrule column for recurring events.
async fn migrate_schedule_blocks_rrule(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if rrule column exists
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT cid, name, type, `notnull`, dflt_value, pk FROM pragma_table_info('schedule_blocks')"
    )
    .fetch_all(pool)
    .await?;

    let has_rrule = columns.iter().any(|(_, name, _, _, _, _)| name == "rrule");

    if !has_rrule {
        info!("Migrating schedule_blocks table: adding rrule column");

        // Add the rrule column (RFC 5545 recurrence rule string)
        sqlx::query("ALTER TABLE schedule_blocks ADD COLUMN rrule TEXT")
            .execute(pool)
            .await?;

        info!("schedule_blocks table migration complete: added rrule column");
    } else {
        debug!("schedule_blocks.rrule column already exists");
    }

    // Add index on rrule for faster recurring block queries
    // This helps with "WHERE rrule IS NOT NULL AND rrule != ''" queries
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_schedule_blocks_rrule ON schedule_blocks(rrule)")
        .execute(pool)
        .await?;

    Ok(())
}
