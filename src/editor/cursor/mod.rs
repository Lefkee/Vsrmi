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
//! **Public API:** [`Position`], [`Cursor`], [`Motion`].

pub mod word;

use crate::editor::document::{Document, indent};

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

/// A cursor movement, independent of the key that triggered it.
///
/// Keeping motions as data rather than as methods is what lets the keymap, the
/// command bar and (later) macros all drive the same movement code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// One character left, wrapping to the previous line.
    Left,
    /// One character right, wrapping to the next line.
    Right,
    /// `n` lines up, keeping the goal column.
    Up(usize),
    /// `n` lines down, keeping the goal column.
    Down(usize),
    /// Start of the next word.
    WordForward,
    /// Start of the previous word.
    WordBackward,
    /// Last character of the current or next word.
    WordEnd,
    /// Column zero of the current line.
    LineStart,
    /// First non-whitespace character of the current line.
    LineFirstNonBlank,
    /// End of the current line.
    LineEnd,
    /// Very start of the document.
    DocStart,
    /// Very end of the document.
    DocEnd,
    /// First non-blank character of a specific line, as used by `:42`.
    ToLine(usize),
}

impl Cursor {
    /// Apply `motion`.
    ///
    /// `extend` leaves the anchor alone (visual mode). `allow_eol` lets the
    /// caret rest one past the final character (insert mode).
    pub fn apply(&mut self, motion: Motion, doc: &Document, extend: bool, allow_eol: bool) {
        match motion {
            Motion::Up(n) => self.move_vertical(doc, n, true, extend, allow_eol),
            Motion::Down(n) => self.move_vertical(doc, n, false, extend, allow_eol),
            Motion::Left => {
                let target = self.left_of(doc);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::Right => {
                let target = self.right_of(doc, allow_eol);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::WordForward => {
                let target = word::next_word_start(doc, self.head);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::WordBackward => {
                let target = word::prev_word_start(doc, self.head);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::WordEnd => {
                let target = word::word_end(doc, self.head);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::LineStart => {
                self.move_to(Position::new(self.head.line, 0), extend);
            }
            Motion::LineFirstNonBlank => {
                let col = indent::first_non_blank(doc, self.head.line);
                let target = Position::new(self.head.line, col);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::LineEnd => {
                let target = Position::new(self.head.line, usize::MAX);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::DocStart => self.move_to(Position::ZERO, extend),
            Motion::DocEnd => {
                let target = Position::new(doc.last_line(), usize::MAX);
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
            Motion::ToLine(line) => {
                let line = line.min(doc.last_line());
                let target = Position::new(line, indent::first_non_blank(doc, line));
                self.move_to(doc.clamp(target, allow_eol), extend);
            }
        }
    }

    /// Vertical movement is the only motion that reads and writes the goal
    /// column, so it does not go through [`Cursor::move_to`].
    fn move_vertical(
        &mut self,
        doc: &Document,
        count: usize,
        up: bool,
        extend: bool,
        allow_eol: bool,
    ) {
        let goal = self.goal_col();
        let line = if up {
            self.head.line.saturating_sub(count)
        } else {
            self.head.line.saturating_add(count).min(doc.last_line())
        };
        self.head = doc.clamp(Position::new(line, goal), allow_eol);
        if !extend {
            self.anchor = self.head;
        }
        self.set_goal_col(goal);
    }

    fn left_of(&self, doc: &Document) -> Position {
        if self.head.col > 0 {
            Position::new(self.head.line, self.head.col - 1)
        } else if self.head.line > 0 {
            let line = self.head.line - 1;
            Position::new(line, doc.line_len(line))
        } else {
            self.head
        }
    }

    fn right_of(&self, doc: &Document, allow_eol: bool) -> Position {
        // The rightmost legal column depends on the mode, so ask the document
        // rather than assuming `line_len`.
        let max_col = doc
            .clamp(Position::new(self.head.line, usize::MAX), allow_eol)
            .col;
        if self.head.col < max_col {
            Position::new(self.head.line, self.head.col + 1)
        } else if self.head.line < doc.last_line() {
            Position::new(self.head.line + 1, 0)
        } else {
            self.head
        }
    }
}
