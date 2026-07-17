//! # User interface
//!
//! **Purpose:** turn application state into a frame.
//!
//! **Responsibility:** split the terminal into regions and hand each one to a
//! widget. The UI layer reads the application state and writes pixels; it never
//! mutates editing state, with the single exception of scrolling the viewport,
//! which cannot be decided until the text area's size is known.
//!
//! **Public API:** [`text::DisplayLine`].

pub mod text;
pub mod widgets;
