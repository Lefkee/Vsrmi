//! # Cursor
//!
//! **Purpose:** describe *where* the editor is looking inside a document.
//!
//! **Responsibility:** owns the text-space coordinate type [`Position`] and,
//! later, the cursor that moves through it. Positions are measured in
//! **characters**, never bytes and never screen columns — byte offsets break on
//! UTF-8 and screen columns break on tabs and wide graphemes, so both
//! conversions are done at the edges (rope access and rendering) instead.
//!
//! **Public API:** [`Position`].

/// A character-wise coordinate inside a document.
///
/// The derived `Ord` compares `line` before `col`, which is exactly document
/// order — that is what makes range normalisation in `selection` a one-liner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Position {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based character offset within the line.
    pub col: usize,
}

impl Position {
    /// The start of the document.
    pub const ZERO: Self = Self { line: 0, col: 0 };

    /// Build a position from a line and character offset.
    #[must_use]
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}
