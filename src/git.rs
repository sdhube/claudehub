//! Git operations and metadata management.
//!
//! This module handles all git-related operations including:
//! - Running git commands with the PDF file's directory as the working directory
//! - Extracting and storing git metadata (commit hashes, blob hashes)
//! - Managing git history for PDF files
//!
//! All git operations are performed with the repository root as the working directory,
//! which is automatically detected based on the PDF file's location.

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::CalibreError;

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

/// Represents git metadata extracted from a file
#[derive(Debug, Clone)]
pub struct ExtractedGitMetadata {
    /// The commit hash of the file's last modification.
    pub commit_hash: String,
    /// The git history (list of commits touching this file).
    pub history: Vec<String>,
    /// The binary hash (blob hash) of the file in Git.
    pub blob_hash: String,
}

/// Manager for git-related database operations and git command execution
pub struct GitManager;

impl GitManager {
    /// Extract git metadata from a file path.
    ///
    /// This function:
    /// 1. Finds the git repository root by looking for .git directory
    /// 2. Runs git commands with the repository root as the working directory
    /// 3. Extracts commit hash, blob hash, and git history
    ///
    /// # Arguments
    ///
    /// * `file_path` - Full path to the PDF file
    ///
    /// # Returns
    ///
    /// Extracted git metadata if the file is in a git repository
    pub fn extract_git_metadata(file_path: &Path) -> Result<ExtractedGitMetadata> {
        // Find the git repository root
        let repo_root = Self::find_git_root(file_path)?;

        // Get relative path from repo root
        let relative_path = file_path
            .strip_prefix(&repo_root)
            .unwrap_or(file_path);

        // Get the commit hash of last modification
        let commit_hash = Self::get_last_commit_hash(&repo_root, relative_path)?;

        // Get the git history
        let history = Self::get_git_history(&repo_root, relative_path)?;

        // Get the blob hash
        let blob_hash = Self::get_blob_hash(&repo_root, relative_path)?;

        Ok(ExtractedGitMetadata {
            commit_hash,
            history,
            blob_hash,
        })
    }

    /// Find the git repository root by checking for .git directory.
    ///
    /// Walks up the directory tree from the given path until it finds a .git directory.
    ///
    /// # Arguments
    ///
    /// * `start_path` - Starting path (usually the PDF file location)
    ///
    /// # Returns
    ///
    /// Path to the git repository root
    fn find_git_root(start_path: &Path) -> Result<PathBuf> {
        let mut current = if start_path.is_file() {
            start_path.parent().unwrap_or(start_path).to_path_buf()
        } else {
            start_path.to_path_buf()
        };

        loop {
            if current.join(".git").exists() {
                tracing::info!("Found git root at: {}", current.display());
                return Ok(current);
            }

            if !current.pop() {
                return Err(CalibreError::InvalidPath(
                    "No Git repository found".to_string(),
                ));
            }
        }
    }

    /// Get the last commit hash for a file using git log.
    ///
    /// Runs: `git log -1 --format=%H <file>`
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the git repository (working directory for git command)
    /// * `relative_path` - Path to the file relative to repo root
    fn get_last_commit_hash(repo_root: &Path, relative_path: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .current_dir(repo_root)
            .args(&["log", "-1", "--format=%H"])
            .arg(relative_path.to_string_lossy().as_ref())
            .output()?;

        if !output.status.success() {
            return Err(CalibreError::InvalidPath(
                "Failed to get git commit hash".to_string(),
            ));
        }

        let hash = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        if hash.is_empty() {
            return Err(CalibreError::InvalidPath(
                "No commit found for file".to_string(),
            ));
        }

        tracing::info!("Got commit hash: {}", hash);
        Ok(hash)
    }

    /// Get the complete git history (list of commit hashes) for a file.
    ///
    /// Runs: `git log --format=%H <file>`
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the git repository (working directory for git command)
    /// * `relative_path` - Path to the file relative to repo root
    fn get_git_history(repo_root: &Path, relative_path: &Path) -> Result<Vec<String>> {
        let output = std::process::Command::new("git")
            .current_dir(repo_root)
            .args(&["log", "--format=%H"])
            .arg(relative_path.to_string_lossy().as_ref())
            .output()?;

        if !output.status.success() {
            tracing::warn!("Failed to get git history");
            return Ok(Vec::new());
        }

        let history = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        tracing::info!("Got {} history entries", history.len());
        Ok(history)
    }

    /// Get the blob hash (object hash) of a file in git.
    ///
    /// Runs: `git hash-object <file>`
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the git repository (working directory for git command)
    /// * `relative_path` - Path to the file relative to repo root
    fn get_blob_hash(repo_root: &Path, relative_path: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .current_dir(repo_root)
            .args(&["hash-object"])
            .arg(relative_path.to_string_lossy().as_ref())
            .output()?;

        if !output.status.success() {
            return Err(CalibreError::InvalidPath(
                "Failed to get git blob hash".to_string(),
            ));
        }

        let hash = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        if hash.is_empty() {
            return Err(CalibreError::InvalidPath(
                "No blob hash found for file".to_string(),
            ));
        }

        tracing::info!("Got blob hash: {}", hash);
        Ok(hash)
    }

    /// Store git metadata for a PDF entry in the database
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

    /// Retrieve git metadata for a PDF entry from the database
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

    /// Update git metadata for a PDF entry in the database
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

    /// Add an entry to git history in the database
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

    /// Retrieve all git history entries for a PDF entry from the database
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pdf_entry_id` - ID of the PDF entry
    ///
    /// # Returns
    ///
    /// Vector of git history entries ordered by commit_order
    pub fn get_git_history_from_db(
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

    /// Clear all git history for a PDF entry from the database
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
        let history = GitManager::get_git_history_from_db(&conn, pdf_entry_id).unwrap();

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
