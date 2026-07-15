//! # Buffer
//!
//! **Purpose:** a document *being edited* — text plus everything the editor
//! remembers about looking at it.
//!
//! **Responsibility:** owns one [`Document`], the cursors pointing into it, and
//! the [`View`] scrolled over it. This is the unit the application opens,
//! switches between and closes; the document below it stays a pure text value.
//!
//! **Public API:** [`Buffer`].

pub mod view;

use crate::editor::cursor::{Cursor, Motion, Position};
use crate::editor::document::Document;

pub use view::View;

/// A document, the cursors editing it, and the viewport showing it.
#[derive(Debug)]
pub struct Buffer {
    /// The text.
    pub document: Document,
    /// Scroll position.
    pub view: View,
    /// Every cursor, kept sorted by head position and never empty.
    ///
    /// Multi-cursor is modelled from the start rather than bolted on: every edit
    /// already iterates this vector, so adding a second cursor later is a UI
    /// change rather than an editing-core change. Document order matters because
    /// edits are applied back-to-front, which keeps earlier offsets valid.
    cursors: Vec<Cursor>,
    /// Index into `cursors` of the one the viewport follows.
    primary: usize,
}

impl Buffer {
    /// Wrap a document in a fresh buffer with a single cursor at the top.
    #[must_use]
    pub fn new(document: Document) -> Self {
        Self {
            document,
            view: View::default(),
            cursors: vec![Cursor::at(Position::ZERO)],
            primary: 0,
        }
    }

    /// An empty scratch buffer.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Document::new())
    }

    /// The primary cursor — the one the viewport follows and the status bar
    /// reports.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursors[self.primary]
    }

    /// Mutable access to the primary cursor.
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[self.primary]
    }

    /// All cursors, in document order.
    #[must_use]
    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }

    /// Whether more than one cursor is active.
    #[must_use]
    pub fn has_multiple_cursors(&self) -> bool {
        self.cursors.len() > 1
    }

    /// Move every cursor by the same motion.
    pub fn move_cursors(&mut self, motion: Motion, extend: bool, allow_eol: bool) {
        for cursor in &mut self.cursors {
            cursor.apply(motion, &self.document, extend, allow_eol);
        }
        self.resort();
    }

    /// Add a secondary cursor, ignoring one that already exists there.
    pub fn add_cursor(&mut self, cursor: Cursor) {
        if self.cursors.iter().any(|c| c.head == cursor.head) {
            return;
        }
        self.cursors.push(cursor);
        self.resort();
    }

    /// Collapse back to a single cursor, keeping the primary one.
    pub fn clear_secondary_cursors(&mut self) {
        let primary = self.cursors[self.primary];
        self.cursors.clear();
        self.cursors.push(primary);
        self.primary = 0;
    }

    /// Indices of all cursors, last in the document first.
    ///
    /// Editing from the end backwards means each edit only shifts text *after*
    /// the cursors already handled, so no offset fix-up pass is needed.
    #[must_use]
    pub fn edit_order(&self) -> Vec<usize> {
        (0..self.cursors.len()).rev().collect()
    }

    /// Mutable access to a cursor by index, used by multi-cursor edits.
    pub fn cursor_at_mut(&mut self, index: usize) -> &mut Cursor {
        &mut self.cursors[index]
    }

    /// Pull every cursor back inside the document.
    ///
    /// Called after any edit that can shrink the text — undo, reload, deleting a
    /// selection — so no cursor is left pointing past the end.
    pub fn clamp_cursors(&mut self, allow_eol: bool) {
        for cursor in &mut self.cursors {
            cursor.head = self.document.clamp(cursor.head, allow_eol);
            cursor.anchor = self.document.clamp(cursor.anchor, allow_eol);
        }
        self.resort();
    }

    /// Restore document order, drop cursors that collided, and follow the
    /// primary one to its new index.
    fn resort(&mut self) {
        if self.cursors.len() == 1 {
            self.primary = 0;
            return;
        }
        let primary_head = self.cursors[self.primary].head;
        self.cursors.sort_by_key(|c| c.head);
        self.cursors.dedup_by_key(|c| c.head);
        self.primary = self
            .cursors
            .iter()
            .position(|c| c.head == primary_head)
            .unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(text: &str) -> Buffer {
        Buffer::new(Document::from_text(text, None))
    }

    #[test]
    fn a_new_buffer_has_exactly_one_cursor() {
        let buf = buffer("abc");
        assert_eq!(buf.cursors().len(), 1);
        assert_eq!(buf.cursor().head, Position::ZERO);
        assert!(!buf.has_multiple_cursors());
    }

    #[test]
    fn cursors_stay_in_document_order() {
        let mut buf = buffer("aaa\nbbb\nccc");
        buf.add_cursor(Cursor::at(Position::new(2, 1)));
        buf.add_cursor(Cursor::at(Position::new(1, 1)));
        let heads: Vec<_> = buf.cursors().iter().map(|c| c.head).collect();
        assert_eq!(
            heads,
            vec![Position::ZERO, Position::new(1, 1), Position::new(2, 1)]
        );
        // The primary cursor followed its position through the sort.
        assert_eq!(buf.cursor().head, Position::ZERO);
    }

    #[test]
    fn duplicate_cursors_are_rejected() {
        let mut buf = buffer("abc");
        buf.add_cursor(Cursor::at(Position::ZERO));
        assert_eq!(buf.cursors().len(), 1);
    }

    #[test]
    fn collapsing_cursors_keeps_the_primary_one() {
        let mut buf = buffer("aaa\nbbb");
        buf.add_cursor(Cursor::at(Position::new(1, 2)));
        buf.clear_secondary_cursors();
        assert_eq!(buf.cursors().len(), 1);
        assert_eq!(buf.cursor().head, Position::ZERO);
    }

    #[test]
    fn merged_cursors_do_not_leave_a_dangling_primary() {
        let mut buf = buffer("abc");
        buf.add_cursor(Cursor::at(Position::new(0, 1)));
        // Both cursors run into the same end-of-line position and merge.
        buf.move_cursors(Motion::LineEnd, false, false);
        assert_eq!(buf.cursors().len(), 1);
        assert_eq!(buf.cursor().head, Position::new(0, 2));
    }

    #[test]
    fn edit_order_runs_backwards_through_the_document() {
        let mut buf = buffer("aaa\nbbb\nccc");
        buf.add_cursor(Cursor::at(Position::new(1, 0)));
        buf.add_cursor(Cursor::at(Position::new(2, 0)));
        assert_eq!(buf.edit_order(), vec![2, 1, 0]);
    }
}
