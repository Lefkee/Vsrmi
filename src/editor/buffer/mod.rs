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
    /// Always non-empty; `cursors[0]` is the primary cursor.
    ///
    /// Multi-cursor is modelled from the start rather than bolted on: every edit
    /// already iterates this vector, so adding a second cursor later is a UI
    /// change rather than an editing-core change.
    cursors: Vec<Cursor>,
}

impl Buffer {
    /// Wrap a document in a fresh buffer with a single cursor at the top.
    #[must_use]
    pub fn new(document: Document) -> Self {
        Self {
            document,
            view: View::default(),
            cursors: vec![Cursor::at(Position::ZERO)],
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
        self.cursors[0]
    }

    /// Mutable access to the primary cursor.
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[0]
    }

    /// All cursors, primary first.
    #[must_use]
    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }

    /// Move every cursor by the same motion.
    pub fn move_cursors(&mut self, motion: Motion, extend: bool, allow_eol: bool) {
        for cursor in &mut self.cursors {
            cursor.apply(motion, &self.document, extend, allow_eol);
        }
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
    }
}
