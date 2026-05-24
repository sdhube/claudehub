//! Database schema definition and initialization.
//!
//! This module defines the SQLite database schema for storing PDF metadata,
//! Git information, and book information. It also provides utilities to
//! create and initialize a new database if one doesn't exist.

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::CalibreError;

/// Database schema version
const SCHEMA_VERSION: u32 = 1;

/// SQL statements for creating the database schema
const SCHEMA_SQL: &str = r#"
-- PDF File Entries Table
CREATE TABLE IF NOT EXISTS pdf_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    book_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Git Metadata Table
CREATE TABLE IF NOT EXISTS git_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pdf_entry_id INTEGER NOT NULL UNIQUE,
    commit_hash TEXT NOT NULL,
    blob_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pdf_entry_id) REFERENCES pdf_entries(id) ON DELETE CASCADE
);

-- Git History Table (many commits per file)
CREATE TABLE IF NOT EXISTS git_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pdf_entry_id INTEGER NOT NULL,
    commit_hash TEXT NOT NULL,
    commit_order INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pdf_entry_id) REFERENCES pdf_entries(id) ON DELETE CASCADE
);

-- Authors Table
CREATE TABLE IF NOT EXISTS authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Books Table
CREATE TABLE IF NOT EXISTS books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pdf_entry_id INTEGER NOT NULL UNIQUE,
    title TEXT NOT NULL,
    author_id INTEGER,
    isbn TEXT,
    description TEXT,
    published_date TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pdf_entry_id) REFERENCES pdf_entries(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE SET NULL
);

-- Tags/Categories Table
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Book Tags Junction Table
CREATE TABLE IF NOT EXISTS book_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    UNIQUE(book_id, tag_id),
    FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_pdf_entries_path ON pdf_entries(path);
CREATE INDEX IF NOT EXISTS idx_pdf_entries_book_name ON pdf_entries(book_name);
CREATE INDEX IF NOT EXISTS idx_git_metadata_pdf_entry ON git_metadata(pdf_entry_id);
CREATE INDEX IF NOT EXISTS idx_git_history_pdf_entry ON git_history(pdf_entry_id);
CREATE INDEX IF NOT EXISTS idx_books_pdf_entry ON books(pdf_entry_id);
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author_id);
CREATE INDEX IF NOT EXISTS idx_book_tags_book ON book_tags(book_id);
CREATE INDEX IF NOT EXISTS idx_book_tags_tag ON book_tags(tag_id);
"#;

/// Manager for database initialization and schema operations.
pub struct SchemaManager;

impl SchemaManager {
    /// Get or create a database in the current directory.
    ///
    /// If a database file doesn't exist at the default location,
    /// it will be created with the proper schema.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Name of the database file (default: "pdf_library.db")
    ///
    /// # Returns
    ///
    /// A tuple containing the database connection and the path to the database file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use calibre_db::schema::SchemaManager;
    /// use std::env;
    ///
    /// let (conn, db_path) = SchemaManager::get_or_create_db(None)?;
    /// println!("Database created at: {}", db_path.display());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_or_create_db(db_name: Option<&str>) -> Result<(Connection, PathBuf)> {
        let db_name = db_name.unwrap_or("pdf_library.db");
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(db_name);

        let db_exists = db_path.exists();

        let conn = rusqlite::Connection::open(&db_path)?;

        if !db_exists {
            Self::initialize_schema(&conn)?;
            tracing::info!("Created new database at: {}", db_path.display());
        } else {
            // Check if schema needs migration
            Self::check_and_migrate_schema(&conn)?;
            tracing::info!("Using existing database at: {}", db_path.display());
        }

        Ok((conn, db_path))
    }

    /// Initialize the database schema.
    fn initialize_schema(conn: &Connection) -> Result<()> {
        // Execute schema SQL
        conn.execute_batch(SCHEMA_SQL)?;

        // Set the schema version
        conn.pragma_update(rusqlite::OptionalExtension::ignore_duplicate_tables, "user_version", SCHEMA_VERSION)?;

        tracing::info!("Database schema initialized (version: {})", SCHEMA_VERSION);
        Ok(())
    }

    /// Check and perform any necessary schema migrations.
    fn check_and_migrate_schema(conn: &Connection) -> Result<()> {
        let current_version: u32 = conn.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0),
        )?;

        if current_version < SCHEMA_VERSION {
            tracing::info!(
                "Migrating database schema from version {} to {}",
                current_version,
                SCHEMA_VERSION
            );
            Self::perform_migrations(conn, current_version)?;
        }

        Ok(())
    }

    /// Perform necessary schema migrations.
    fn perform_migrations(_conn: &Connection, _from_version: u32) -> Result<()> {
        // Future migrations can be added here
        // For now, just log that no migrations are needed
        tracing::debug!("No migrations needed");
        Ok(())
    }

    /// Create a database at a specific path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the database should be created
    ///
    /// # Returns
    ///
    /// A connection to the newly created database.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use calibre_db::schema::SchemaManager;
    ///
    /// let conn = SchemaManager::create_db_at("/path/to/database.db")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_db_at<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let path = path.as_ref();

        if path.exists() {
            return Err(CalibreError::InvalidPath(
                format!("Database file already exists at: {}", path.display()),
            ));
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = rusqlite::Connection::open(path)?;
        Self::initialize_schema(&conn)?;

        tracing::info!("Created new database at: {}", path.display());
        Ok(conn)
    }

    /// Drop all tables from the database (useful for testing).
    ///
    /// # Warning
    ///
    /// This will delete all data. Use with caution!
    pub fn drop_all_tables(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            DROP TABLE IF EXISTS book_tags;
            DROP TABLE IF EXISTS tags;
            DROP TABLE IF EXISTS books;
            DROP TABLE IF EXISTS authors;
            DROP TABLE IF EXISTS git_history;
            DROP TABLE IF EXISTS git_metadata;
            DROP TABLE IF EXISTS pdf_entries;
            "#,
        )?;

        tracing::warn!("All tables dropped from database");
        Ok(())
    }

    /// Get the schema version of the database.
    pub fn get_schema_version(conn: &Connection) -> Result<u32> {
        let version: u32 = conn.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0),
        )?;

        Ok(version)
    }

    /// Verify that all expected tables exist in the database.
    pub fn verify_schema(conn: &Connection) -> Result<bool> {
        let tables = vec![
            "pdf_entries",
            "git_metadata",
            "git_history",
            "authors",
            "books",
            "tags",
            "book_tags",
        ];

        for table in tables {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )?;

            if !exists {
                tracing::warn!("Expected table '{}' not found in database", table);
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_db_at_new_location() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = SchemaManager::create_db_at(&db_path).unwrap();
        assert!(db_path.exists());

        let version = SchemaManager::get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        let schema_valid = SchemaManager::verify_schema(&conn).unwrap();
        assert!(schema_valid);
    }

    #[test]
    fn test_create_db_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create first database
        SchemaManager::create_db_at(&db_path).unwrap();

        // Attempt to create at same path should fail
        let result = SchemaManager::create_db_at(&db_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_schema() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = SchemaManager::create_db_at(&db_path).unwrap();
        let schema_valid = SchemaManager::verify_schema(&conn).unwrap();
        assert!(schema_valid);
    }
}
