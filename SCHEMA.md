# Calibre Database Schema

This document describes the SQLite database schema used by Calibre for storing ebook metadata and library information.

## Overview

Calibre stores all library metadata in an SQLite database (typically named `metadata.db`). The schema is designed to efficiently manage books, authors, series, tags, and other related metadata.

## Core Tables

### `books`
Main table containing book metadata.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique book identifier |
| `title` | TEXT NOT NULL | Book title |
| `sort` | TEXT | Sortable version of title |
| `timestamp` | TIMESTAMP | When book was added to library |
| `pubdate` | TIMESTAMP | Publication date |
| `series_index` | REAL | Position in series (if applicable) |
| `author_sort` | TEXT | Sorted author name(s) |
| `path` | TEXT NOT NULL | Path within Calibre library |
| `uuid` | TEXT | Unique identifier (UUID4) |
| `has_cover` | BOOLEAN | Whether book has a cover image |
| `last_modified` | TIMESTAMP | Last modification time |

### `authors`
Stores author information.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique author identifier |
| `name` | TEXT | Author name |
| `sort` | TEXT | Sortable author name |
| `link` | TEXT | Author link/URL |

### `series`
Stores series information.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique series identifier |
| `name` | TEXT | Series name |
| `sort` | TEXT | Sortable series name |
| `link` | TEXT | Series link/URL |

### `tags`
Stores tag/category information.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique tag identifier |
| `name` | TEXT | Tag name |
| `link` | TEXT | Tag link/URL |

### `publishers`
Stores publisher information.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique publisher identifier |
| `name` | TEXT | Publisher name |
| `link` | TEXT | Publisher link/URL |

### `ratings`
Stores book ratings.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique rating identifier |
| `rating` | INTEGER | Rating value (0-10 typically) |
| `link` | TEXT | Rating link/URL |

### `languages`
Stores language information.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique language identifier |
| `lang_code` | TEXT NOT NULL UNIQUE | Language code (e.g., 'en', 'fr') |
| `link` | TEXT | Language link/URL |

### `data`
Stores book format information and file metadata.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `format` | TEXT NOT NULL | Format code (e.g., 'EPUB', 'PDF', 'MOBI') |
| `uncompressed_size` | INTEGER | Uncompressed file size in bytes |

### `comments`
Stores book descriptions and comments.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL UNIQUE | Foreign key to `books.id` |
| `text` | TEXT | Comment/description text |

### `identifiers`
Stores book identifiers (ISBN, DOI, etc.).

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `type` | TEXT DEFAULT "isbn" | Identifier type (e.g., 'isbn', 'doi') |
| `val` | TEXT NOT NULL | Identifier value |

### `custom_columns`
Defines custom metadata columns.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `label` | TEXT NOT NULL UNIQUE | Column label/key |
| `name` | TEXT NOT NULL | Display name |
| `datatype` | TEXT NOT NULL | Data type (text, series, enumeration, etc.) |
| `mark_for_delete` | BOOLEAN | Mark for deletion |
| `editable` | BOOLEAN | Whether column is user-editable |
| `display` | TEXT | Display settings JSON |
| `is_multiple` | BOOLEAN | Can store multiple values |
| `normalized` | BOOLEAN | Whether values are normalized |

## Many-to-Many Junction Tables

These tables establish relationships between books and other entities:

### `books_authors_link`
Links books to authors.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `author` | INTEGER NOT NULL | Foreign key to `authors.id` |

### `books_tags_link`
Links books to tags.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `tag` | INTEGER NOT NULL | Foreign key to `tags.id` |

### `books_series_link`
Links books to series.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `series` | INTEGER NOT NULL | Foreign key to `series.id` |

### `books_publishers_link`
Links books to publishers.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `publisher` | INTEGER NOT NULL | Foreign key to `publishers.id` |

### `books_ratings_link`
Links books to ratings.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `rating` | INTEGER NOT NULL | Foreign key to `ratings.id` |

### `books_languages_link`
Links books to languages.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `lang_code` | INTEGER NOT NULL | Foreign key to `languages.id` |
| `item_order` | INTEGER DEFAULT 0 | Order of language |

## Specialized Tables

### `library_id`
Stores the unique identifier for the library.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `uuid` | TEXT NOT NULL UNIQUE | Library UUID |

### `books_plugin_data`
Stores custom data from plugins.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `name` | TEXT NOT NULL | Plugin data name |
| `val` | TEXT NOT NULL | Plugin data value |

### `metadata_dirtied`
Tracks books with metadata changes needing OPF backup.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL UNIQUE | Foreign key to `books.id` |

### `last_read_positions`
Stores reading progress for ebooks.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `format` | TEXT NOT NULL | Book format (EPUB, PDF, etc.) |
| `user` | TEXT NOT NULL | User identifier |
| `device` | TEXT NOT NULL | Device identifier |
| `cfi` | TEXT NOT NULL | EPUB CFI position |
| `epoch` | REAL NOT NULL | Timestamp |
| `pos_frac` | REAL DEFAULT 0 | Position fraction (0-1) |

### `annotations`
Stores user annotations and highlights.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL | Foreign key to `books.id` |
| `format` | TEXT NOT NULL | Book format |
| `user_type` | TEXT NOT NULL | Type of user (local, remote, etc.) |
| `user` | TEXT NOT NULL | User identifier |
| `timestamp` | REAL NOT NULL | Annotation timestamp |
| `annot_id` | TEXT NOT NULL | Unique annotation identifier |
| `annot_type` | TEXT NOT NULL | Type of annotation (highlight, bookmark, etc.) |
| `annot_data` | TEXT NOT NULL | Annotation data JSON |
| `searchable_text` | TEXT | Searchable annotation text |

### `annotations_dirtied`
Tracks annotations needing sync.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique identifier |
| `book` | INTEGER NOT NULL UNIQUE | Foreign key to `books.id` |

### `books_pages_link`
Stores page count information.

| Column | Type | Description |
|--------|------|-------------|
| `book` | INTEGER PRIMARY KEY | Foreign key to `books.id` |
| `pages` | INTEGER DEFAULT 0 | Page count |
| `algorithm` | INTEGER DEFAULT 0 | Algorithm used to calculate pages |
| `format` | TEXT DEFAULT '' | Format used for page detection |
| `format_size` | INTEGER DEFAULT 0 | Format file size |
| `timestamp` | TIMESTAMP | Last update timestamp |
| `needs_scan` | INTEGER | Whether needs rescanning |

## Views

### `meta`
Aggregated view of book metadata combining data from multiple tables.

### `tag_browser_*` Views
Various tag browser views (authors, tags, publishers, series) with book counts and average ratings.

## Indexes

The schema includes numerous indexes for performance optimization:

- `authors_idx` - ON books (author_sort, sort)
- `series_idx` - ON series (name)
- `series_sort_idx` - ON books (series_index, id)
- `books_idx` - ON books (sort)
- `formats_idx` - ON data (format)
- `languages_idx` - ON languages (lang_code)
- `annot_idx` - ON annotations (book)
- `lrp_idx` - ON last_read_positions (book)
- Various link table indexes for foreign key relationships

## Triggers

The schema defines several triggers for data integrity:

- **books_delete_trg** - Cascading deletes when a book is deleted
- **books_insert_trg** - Auto-generates sort field and UUID on insert
- **books_update_trg** - Updates sort field when title changes
- **series_insert_trg/series_update_trg** - Updates series sort field
- **Foreign key triggers** - Prevent invalid references in junction tables
- **Annotation FTS triggers** - Maintains full-text search indexes

## Full-Text Search (FTS)

The `annotations_fts` and `annotations_fts_stemmed` virtual tables provide full-text search capabilities for annotations using SQLite's FTS5 module.

## Schema Versioning

The database uses `PRAGMA user_version` to track schema version for migrations. Current versions track database evolution from v0 through v26, with each upgrade potentially adding or modifying tables and triggers.

## Relationships Diagram

```
books
├── books_authors_link ── authors
├── books_tags_link ────── tags
├── books_series_link ──── series
├── books_publishers_link ─ publishers
├── books_ratings_link ──── ratings
├── books_languages_link ── languages
├── data
├── comments
├── identifiers
├── custom_columns
├── books_plugin_data
├── last_read_positions
├── annotations
└── books_pages_link
```

## References

- [Calibre Official Documentation](https://calibre-ebook.com/)
- [Calibre Database API](https://manual.calibre-ebook.com/db_api.html)
- [Calibre GitHub Repository](https://github.com/kovidgoyal/calibre)
