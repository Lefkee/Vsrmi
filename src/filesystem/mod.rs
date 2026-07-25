//! # Filesystem
//!
//! **Purpose:** every read from and write to disk goes through here.
//!
//! **Responsibility:** turn raw I/O into `anyhow` errors carrying the path (a
//! bare "access denied" is useless in a status bar), and make saving atomic so
//! that a crash mid-write cannot leave the user with a truncated source file.
//!
//! **Public API:** [`read_file`], [`write_file`], [`tree::Tree`].

pub mod tree;
pub mod watcher;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Read a UTF-8 file into memory.
///
/// # Errors
/// Returns an error if the file cannot be read or is not valid UTF-8.
pub fn read_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    String::from_utf8(bytes).with_context(|| format!("{} is not valid UTF-8", path.display()))
}

/// Write `contents` to `path` atomically.
///
/// The data lands in a sibling temporary file first and is then renamed over the
/// target. `rename` within a directory is atomic on every platform we support,
/// so a reader either sees the old file or the new one — never a half-written
/// one.
///
/// # Errors
/// Returns an error if the temporary file cannot be written or renamed.
pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    let temp = temp_path(path);
    fs::write(&temp, contents).with_context(|| format!("cannot write {}", temp.display()))?;
    fs::rename(&temp, path)
        .with_context(|| format!("cannot replace {}", path.display()))
        .inspect_err(|_| {
            // Do not leave debris behind if the rename failed.
            let _ = fs::remove_file(&temp);
        })
}

/// A sibling path for the staging file used by [`write_file`].
///
/// It must live in the same directory as the target, otherwise the rename would
/// cross a filesystem boundary and stop being atomic.
fn temp_path(path: &Path) -> std::path::PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().unwrap_or_else(|| "termi".as_ref()));
    name.push(".termi-tmp");
    path.with_file_name(name)
}
