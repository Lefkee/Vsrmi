//! # Widgets
//!
//! **Purpose:** one module per region of the screen.
//!
//! **Responsibility:** each widget borrows exactly the state it draws and
//! implements [`ratatui::widgets::Widget`]. None of them mutate the editor, so
//! the whole render pass is a pure function of the application state plus the
//! terminal size.
//!
//! **Public API:** the widget types re-exported below.

pub mod editor_view;

pub use editor_view::EditorView;
