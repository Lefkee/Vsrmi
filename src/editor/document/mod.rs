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

use ropey::{Rope, RopeSlice};

use crate::editor::cursor::Position;

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

/// Read-only text access.
///
/// Every method here clamps its arguments instead of panicking. Cursors,
/// viewports and search results all index the rope, and a document can shrink
/// underneath any of them (undo, external reload); clamping turns a whole class
/// of races into a harmless off-by-a-few instead of a crash.
impl Document {
    /// Total number of characters, excluding nothing.
    #[must_use]
    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    /// Number of lines. A trailing newline yields a final empty line, matching
    /// what the user sees.
    #[must_use]
    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    /// Index of the last line.
    #[must_use]
    pub fn last_line(&self) -> usize {
        self.text.len_lines() - 1
    }

    /// A line including its terminator, clamped to the document.
    #[must_use]
    pub fn line(&self, line: usize) -> RopeSlice<'_> {
        self.text.line(line.min(self.last_line()))
    }

    /// Length of a line in characters, *excluding* the line terminator.
    #[must_use]
    pub fn line_len(&self, line: usize) -> usize {
        line_content_len(self.line(line))
    }

    /// Character index of the first character on `line`.
    #[must_use]
    pub fn line_start(&self, line: usize) -> usize {
        self.text.line_to_char(line.min(self.last_line()))
    }

    /// Convert a position into a rope character index.
    #[must_use]
    pub fn pos_to_char(&self, pos: Position) -> usize {
        let line = pos.line.min(self.last_line());
        self.text.line_to_char(line) + pos.col.min(self.line_len(line))
    }

    /// Convert a rope character index back into a position.
    #[must_use]
    pub fn char_to_pos(&self, index: usize) -> Position {
        let index = index.min(self.text.len_chars());
        let line = self.text.char_to_line(index);
        Position::new(line, index - self.text.line_to_char(line))
    }

    /// The character at `pos`, or `None` at end of line or end of file.
    #[must_use]
    pub fn char_at(&self, pos: Position) -> Option<char> {
        let line = pos.line.min(self.last_line());
        (pos.col < self.line_len(line)).then(|| self.text.char(self.line_start(line) + pos.col))
    }

    /// Pull a position back inside the document.
    ///
    /// `allow_eol` is `true` in insert mode, where the cursor legitimately sits
    /// one past the last character, and `false` in normal mode, where the cursor
    /// always covers a real character.
    #[must_use]
    pub fn clamp(&self, pos: Position, allow_eol: bool) -> Position {
        let line = pos.line.min(self.last_line());
        let len = self.line_len(line);
        let max_col = if allow_eol { len } else { len.saturating_sub(1) };
        Position::new(line, pos.col.min(max_col))
    }

    /// Copy a line into a `String`, without its terminator.
    #[must_use]
    pub fn line_string(&self, line: usize) -> String {
        let slice = self.line(line);
        slice.slice(..line_content_len(slice)).to_string()
    }
}

/// Length of a line slice with any `\r\n` / `\n` terminator removed.
fn line_content_len(slice: RopeSlice<'_>) -> usize {
    let mut len = slice.len_chars();
    if len > 0 && slice.char(len - 1) == '\n' {
        len -= 1;
    }
    if len > 0 && slice.char(len - 1) == '\r' {
        len -= 1;
    }
    len
}
