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
//! **Public API:** [`Position`], [`Cursor`].

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

/// One insertion point, optionally dragging a selection behind it.
///
/// `head` is where the caret is and where text is inserted; `anchor` is where
/// the selection started. When the two are equal there is no selection. Modelling
/// it this way — rather than as an `Option<Selection>` beside the cursor — means
/// a multi-cursor edit is just the same operation run over a `Vec<Cursor>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    /// The moving end of the selection; the caret.
    pub head: Position,
    /// The fixed end of the selection.
    pub anchor: Position,
    /// Column the cursor "wants" to be in during vertical movement.
    ///
    /// Moving down from a long line through a short one and back must return to
    /// the original column, so vertical motions remember the goal instead of
    /// reading the clamped column back out. `None` means "use the current
    /// column", and any horizontal motion resets it.
    goal_col: Option<usize>,
}

impl Cursor {
    /// A collapsed cursor at `pos`.
    #[must_use]
    pub const fn at(pos: Position) -> Self {
        Self {
            head: pos,
            anchor: pos,
            goal_col: None,
        }
    }

    /// Whether the cursor currently spans any text.
    #[must_use]
    pub fn has_selection(&self) -> bool {
        self.head != self.anchor
    }

    /// Drop the selection, leaving the caret where it is.
    pub fn collapse(&mut self) {
        self.anchor = self.head;
    }

    /// Start a selection at the current caret.
    pub fn anchor_here(&mut self) {
        self.anchor = self.head;
    }

    /// Move the caret, dropping the remembered goal column.
    ///
    /// `extend` keeps the anchor in place (visual mode); otherwise the selection
    /// collapses onto the new position.
    pub fn move_to(&mut self, pos: Position, extend: bool) {
        self.head = pos;
        self.goal_col = None;
        if !extend {
            self.anchor = pos;
        }
    }

    /// The column vertical movement should aim for.
    #[must_use]
    pub fn goal_col(&self) -> usize {
        self.goal_col.unwrap_or(self.head.col)
    }

    /// Remember a goal column across a run of vertical motions.
    pub fn set_goal_col(&mut self, col: usize) {
        self.goal_col = Some(col);
    }
}
