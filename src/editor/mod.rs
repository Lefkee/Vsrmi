//! # Editor core
//!
//! **Purpose:** everything that manipulates text, with no knowledge of
//! terminals, keys or themes.
//!
//! **Responsibility:** this layer is deliberately UI-free so it stays testable
//! and reusable. It is split by concern:
//!
//! - [`document`] — the text itself, its file and its dirty state
//! - [`cursor`] — text coordinates and motions
//! - [`selection`] — ranges anchored to a cursor
//! - [`buffer`] — a document plus the cursors and viewport looking at it
//! - [`command`] — ex-style commands typed into the command bar

pub mod cursor;
pub mod document;
