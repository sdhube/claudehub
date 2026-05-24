//! PDF scanning module for recursive directory traversal.
//!
//! This module provides functionality to scan a directory recursively, find PDF files,
//! extract metadata (including Git information when available), and add entries to the database.

use std::fs;
use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::CalibreError;

/// Git metadata for a file.
#[derive(Debug, Clone)]
pub struct GitMetadata {
    /// The commit hash of the file's last modification.
    pub commit_hash: String,
    /// The git history (list of commits touching this file).
    pub history: Vec<String>,
    /// The binary hash (blob hash) of the file in Git.
    pub blob_hash: String,
}

/// Metadata for a PDF file found during scanning.
#[derive(Debug, Clone)]
pub struct PdfEntry {
    /// Full path to the PDF file.
    pub path: PathBuf,
    /// Name of the book (derived from filename or metadata).
    pub book_name: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Git metadata (if file is in a Git repository).
    pub git_metadata: Option<GitMetadata>,
}

/// Scanner for finding and cataloging PDF files.
pub struct PdfScanner {
    /// Root directory to scan.
    root_dir: PathBuf,
}

impl PdfScanner {
    /// Create a new PDF scanner for the given directory.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The directory to start scanning from
    ///
    /// # Example
    ///
    /// ```no_run
    /// use calibre_db::scan::PdfScanner;
    ///
    /// let scanner = PdfScanner::new("/path/to/books")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        
        if !root_dir.exists() {
            return Err(CalibreError::InvalidPath(
                format!("Directory not found: {}", root_dir.display()),
            ));
        }
        
        if !root_dir.is_dir() {
            return Err(CalibreError::InvalidPath(
                format!("Path is not a directory: {}", root_dir.display()),
            ));
        }
        
        Ok(PdfScanner { root_dir })
    }
    
    /// Scan the directory recursively and find all PDF files.
    ///
    /// # Returns
    ///
    /// A vector of `PdfEntry` objects containing metadata for each found PDF.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use calibre_db::scan::PdfScanner;
    ///
    /// let scanner = PdfScanner::new("/path/to/books")?;
    /// let pdfs = scanner.scan()?;
    /// println!("Found {} PDFs", pdfs.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn scan(&self) -> Result<Vec<PdfEntry>> {
        let mut entries = Vec::new();
        self.scan_recursive(&self.root_dir, &mut entries)?;
        Ok(entries)
    }
    
    /// Recursively scan a directory for PDF files.
    fn scan_recursive(&self, dir: &Path, entries: &mut Vec<PdfEntry>) -> Result<()> {
        match fs::read_dir(dir) {
            Ok(dir_entries) => {
                for entry in dir_entries {
                    match entry {
                        Ok(dir_entry) => {
                            let path = dir_entry.path();
                            
                            if path.is_dir() {
                                // Recursively scan subdirectories
                                self.scan_recursive(&path, entries)?;
                            } else if path.is_file() {
                                // Check if file is a PDF
                                if let Some(ext) = path.extension() {
                                    if ext.eq_ignore_ascii_case("pdf") {
                                        if let Ok(pdf_entry) = self.create_pdf_entry(&path) {
                                            entries.push(pdf_entry);
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Skip entries that can't be read
                            continue;
                        }
                    }
                }
            }
            Err(_) => {
                // Skip directories that can't be read
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Create a PDF entry for a file, including Git metadata if available.
    fn create_pdf_entry(&self, path: &Path) -> Result<PdfEntry> {
        let file_size = fs::metadata(path)?
            .len();
        
        let book_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let git_metadata = self.get_git_metadata(path).ok();
        
        Ok(PdfEntry {
            path: path.to_path_buf(),
            book_name,
            file_size,
            git_metadata,
        })
    }
    
    /// Get Git metadata for a file if it's in a Git repository.
    ///
    /// This function attempts to find the Git repository root and extract:
    /// - The commit hash of the last modification
    /// - The Git history (list of commit hashes)
    /// - The blob hash (binary hash) of the file
    fn get_git_metadata(&self, file_path: &Path) -> Result<GitMetadata> {
        // Try to find the Git repository root
        let repo_root = self.find_git_root(file_path)?;
        
        // Get relative path from repo root
        let relative_path = file_path
            .strip_prefix(&repo_root)
            .unwrap_or(file_path);
        
        // Get the commit hash of last modification
        let commit_hash = self.get_last_commit_hash(&repo_root, relative_path)?;
        
        // Get the Git history
        let history = self.get_git_history(&repo_root, relative_path)?;
        
        // Get the blob hash
        let blob_hash = self.get_blob_hash(&repo_root, relative_path)?;
        
        Ok(GitMetadata {
            commit_hash,
            history,
            blob_hash,
        })
    }
    
    /// Find the Git repository root by checking for .git directory.
    fn find_git_root(&self, start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.to_path_buf();
        
        loop {
            if current.join(".git").exists() {
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
    fn get_last_commit_hash(&self, repo_root: &Path, relative_path: &Path) -> Result<String> {
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
        
        Ok(hash)
    }
    
    /// Get the complete Git history (list of commit hashes) for a file.
    fn get_git_history(&self, repo_root: &Path, relative_path: &Path) -> Result<Vec<String>> {
        let output = std::process::Command::new("git")
            .current_dir(repo_root)
            .args(&["log", "--format=%H"])
            .arg(relative_path.to_string_lossy().as_ref())
            .output()?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let history = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        
        Ok(history)
    }
    
    /// Get the blob hash (binary hash) of a file in Git.
    fn get_blob_hash(&self, repo_root: &Path, relative_path: &Path) -> Result<String> {
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
        
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pdf_scanner_invalid_directory() {
        let result = PdfScanner::new("/nonexistent/path");
        assert!(result.is_err());
    }
}
