//! # Document
//!
//! **Purpose:** hold one file's text and the metadata that belongs to the text
//! rather than to the view of it.
//!
//! **Responsibility:** storage (a [`Rope`]), the originating path, the line
//! ending style, and the dirty flag. A `Document` never knows where the cursor
//! is or how it is displayed — that is [`crate::editor::buffer`]'s job.
//!
//! Line endings are normalised to `\n` on load and restored on save. Doing it
//! at the edges means every offset in the editor is a plain character index
//! with no invisible `\r` to account for.
//!
//! **Public API:** [`Document`], [`LineEnding`].

use std::path::{Path, PathBuf};

use ropey::Rope;

/// The line terminator a file uses on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    /// Unix `\n`.
    #[default]
    Lf,
    /// Windows `\r\n`.
    Crlf,
}

impl LineEnding {
    /// Guess the line ending from the first terminator in `text`.
    ///
    /// Mixed files are common; the first one wins and the file is normalised to
    /// it on save.
    #[must_use]
    pub fn detect(text: &str) -> Self {
        match text.find('\n') {
            Some(idx) if idx > 0 && text.as_bytes()[idx - 1] == b'\r' => Self::Crlf,
            _ => Self::Lf,
        }
    }

    /// The characters written to disk for this line ending.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }

    /// Short label for the status bar.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
        }
    }
}

/// A single open file's text.
#[derive(Debug)]
pub struct Document {
    text: Rope,
    path: Option<PathBuf>,
    line_ending: LineEnding,
    dirty: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// An empty, unnamed document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: Rope::new(),
            path: None,
            line_ending: LineEnding::default(),
            dirty: false,
        }
    }

    /// Build a document from in-memory text, normalising line endings.
    #[must_use]
    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        let line_ending = LineEnding::detect(text);
        let normalised = if line_ending == LineEnding::Crlf {
            Rope::from_str(&text.replace("\r\n", "\n"))
        } else {
            Rope::from_str(text)
        };
        Self {
            text: normalised,
            path,
            line_ending,
            dirty: false,
        }
    }

    /// The underlying rope, for read-only traversal.
    #[must_use]
    pub const fn text(&self) -> &Rope {
        &self.text
    }

    /// The file this document came from, if any.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Replace the associated path, as done by `:w <name>`.
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// The line ending this document will be written with.
    #[must_use]
    pub const fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// File name for the tab strip and status bar.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("[No Name]")
    }
}
