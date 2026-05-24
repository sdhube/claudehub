//! Git operations and metadata management.
//!
//! This module handles all git-related operations including storing and retrieving
//! git metadata (commit hashes, blob hashes) and git history for PDF files.

use rusqlite::Connection;
use crate::error::Result;

/// Represents git metadata for a PDF file
#[derive(Debug, Clone)]
pub struct GitMetadata {
    pub id: i64,
    pub pdf_entry_id: i64,
    pub commit_hash: String,
    pub blob_hash: String,
}

/// Represents a git history entry for a PDF file
#[derive(Debug, Clone)]
pub struct GitHistoryEntry {
    pub id: i64,
    pub pdf_entry_id: i64,
    pub commit_hash: String,
    pub commit_order: i32,
}

/// Manager for git-related database operations
pub struct GitManager;

impl GitManager {
    /// Store git metadata for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    /// * `commit_hash` - Git commit hash
    /// * `blob_hash` - Git blob hash
    ///
    /// # Returns
    ///
    /// The ID of the inserted git metadata record
    pub fn store_git_metadata(
        conn: &Connection,
        pdf_entry_id: i64,
        commit_hash: &str,
        blob_hash: &str,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO git_metadata (pdf_entry_id, commit_hash, blob_hash)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![pdf_entry_id, commit_hash, blob_hash],
        )?;

        let id = conn.last_insert_rowid();
        tracing::info!(
            "Stored git metadata for pdf_entry_id: {} (metadata_id: {})",
            pdf_entry_id,
            id
        );

        Ok(id)
    }

    /// Retrieve git metadata for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    ///
    /// # Returns
    ///
    /// Optional git metadata if found
    pub fn get_git_metadata(
        conn: &Connection,
        pdf_entry_id: i64,
    ) -> Result<Option<GitMetadata>> {
        let mut stmt = conn.prepare(
            "SELECT id, pdf_entry_id, commit_hash, blob_hash
             FROM git_metadata
             WHERE pdf_entry_id = ?1",
        )?;

        let result = stmt.query_row(rusqlite::params![pdf_entry_id], |row| {
            Ok(GitMetadata {
                id: row.get(0)?,
                pdf_entry_id: row.get(1)?,
                commit_hash: row.get(2)?,
                blob_hash: row.get(3)?,
            })
        });

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update git metadata for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    /// * `commit_hash` - New git commit hash
    /// * `blob_hash` - New git blob hash
    pub fn update_git_metadata(
        conn: &Connection,
        pdf_entry_id: i64,
        commit_hash: &str,
        blob_hash: &str,
    ) -> Result<()> {
        conn.execute(
            "UPDATE git_metadata
             SET commit_hash = ?1, blob_hash = ?2
             WHERE pdf_entry_id = ?3",
            rusqlite::params![commit_hash, blob_hash, pdf_entry_id],
        )?;

        tracing::info!("Updated git metadata for pdf_entry_id: {}", pdf_entry_id);
        Ok(())
    }

    /// Add an entry to git history
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    /// * `commit_hash` - Git commit hash
    /// * `commit_order` - Order of the commit in the history
    ///
    /// # Returns
    ///
    /// The ID of the inserted git history record
    pub fn add_git_history(
        conn: &Connection,
        pdf_entry_id: i64,
        commit_hash: &str,
        commit_order: i32,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO git_history (pdf_entry_id, commit_hash, commit_order)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![pdf_entry_id, commit_hash, commit_order],
        )?;

        let id = conn.last_insert_rowid();
        tracing::info!(
            "Added git history entry for pdf_entry_id: {} (history_id: {})",
            pdf_entry_id,
            id
        );

        Ok(id)
    }

    /// Retrieve all git history entries for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    ///
    /// # Returns
    ///
    /// Vector of git history entries ordered by commit_order
    pub fn get_git_history(
        conn: &Connection,
        pdf_entry_id: i64,
    ) -> Result<Vec<GitHistoryEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, pdf_entry_id, commit_hash, commit_order
             FROM git_history
             WHERE pdf_entry_id = ?1
             ORDER BY commit_order ASC",
        )?;

        let entries = stmt.query_map(rusqlite::params![pdf_entry_id], |row| {
            Ok(GitHistoryEntry {
                id: row.get(0)?,
                pdf_entry_id: row.get(1)?,
                commit_hash: row.get(2)?,
                commit_order: row.get(3)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// Clear all git history for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    pub fn clear_git_history(conn: &Connection, pdf_entry_id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM git_history WHERE pdf_entry_id = ?1",
            rusqlite::params![pdf_entry_id],
        )?;

        tracing::info!("Cleared git history for pdf_entry_id: {}", pdf_entry_id);
        Ok(())
    }

    /// Get the count of git history entries for a PDF entry
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    ///
    /// # Returns
    ///
    /// Number of history entries
    pub fn get_history_count(conn: &Connection, pdf_entry_id: i64) -> Result<i64> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM git_history WHERE pdf_entry_id = ?1",
            rusqlite::params![pdf_entry_id],
            |row| row.get(0),
        )?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaManager;
    use tempfile::TempDir;

    fn create_test_db() -> (Connection, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = SchemaManager::create_db_at(&db_path).unwrap();
        (conn, temp_dir)
    }

    #[test]
    fn test_store_and_get_git_metadata() {
        let (conn, _temp) = create_test_db();

        // Create a PDF entry first
        conn.execute(
            "INSERT INTO pdf_entries (path, book_name, file_size) VALUES (?1, ?2, ?3)",
            rusqlite::params!["/test/file.pdf", "Test Book", 1024],
        )
        .unwrap();

        let pdf_entry_id = conn.last_insert_rowid();

        // Store git metadata
        let metadata_id = GitManager::store_git_metadata(
            &conn,
            pdf_entry_id,
            "abc123",
            "def456",
        )
        .unwrap();

        assert!(metadata_id > 0);

        // Retrieve and verify
        let metadata = GitManager::get_git_metadata(&conn, pdf_entry_id)
            .unwrap()
            .unwrap();

        assert_eq!(metadata.pdf_entry_id, pdf_entry_id);
        assert_eq!(metadata.commit_hash, "abc123");
        assert_eq!(metadata.blob_hash, "def456");
    }

    #[test]
    fn test_add_and_get_git_history() {
        let (conn, _temp) = create_test_db();

        // Create a PDF entry first
        conn.execute(
            "INSERT INTO pdf_entries (path, book_name, file_size) VALUES (?1, ?2, ?3)",
            rusqlite::params!["/test/file.pdf", "Test Book", 1024],
        )
        .unwrap();

        let pdf_entry_id = conn.last_insert_rowid();

        // Add multiple history entries
        GitManager::add_git_history(&conn, pdf_entry_id, "commit1", 1).unwrap();
        GitManager::add_git_history(&conn, pdf_entry_id, "commit2", 2).unwrap();
        GitManager::add_git_history(&conn, pdf_entry_id, "commit3", 3).unwrap();

        // Retrieve history
        let history = GitManager::get_git_history(&conn, pdf_entry_id).unwrap();

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].commit_hash, "commit1");
        assert_eq!(history[1].commit_hash, "commit2");
        assert_eq!(history[2].commit_hash, "commit3");
    }

    #[test]
    fn test_clear_git_history() {
        let (conn, _temp) = create_test_db();

        // Create a PDF entry
        conn.execute(
            "INSERT INTO pdf_entries (path, book_name, file_size) VALUES (?1, ?2, ?3)",
            rusqlite::params!["/test/file.pdf", "Test Book", 1024],
        )
        .unwrap();

        let pdf_entry_id = conn.last_insert_rowid();

        // Add history
        GitManager::add_git_history(&conn, pdf_entry_id, "commit1", 1).unwrap();
        GitManager::add_git_history(&conn, pdf_entry_id, "commit2", 2).unwrap();

        let count = GitManager::get_history_count(&conn, pdf_entry_id).unwrap();
        assert_eq!(count, 2);

        // Clear history
        GitManager::clear_git_history(&conn, pdf_entry_id).unwrap();

        let count = GitManager::get_history_count(&conn, pdf_entry_id).unwrap();
        assert_eq!(count, 0);
    }
}
