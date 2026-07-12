//! # Application state
//!
//! **Purpose:** the single mutable value that the whole editor operates on.
//!
//! **Responsibility:** holds the editor mode and the "should we keep running"
//! flag. Every subsystem reads from and writes to this struct; nothing else in
//! the editor owns global mutable state.
//!
//! **Public API:** [`App`].

use super::mode::Mode;

/// Editor-wide state.
#[derive(Debug, Default)]
pub struct App {
    /// Current modal state.
    pub mode: Mode,
    /// Set to `true` to leave the event loop after the current iteration.
    quit: bool,
}

impl App {
    /// Create an editor sitting in normal mode.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request shutdown; the event loop exits once the current event is done.
    pub fn quit(&mut self) {
        self.quit = true;
    }

    /// Whether the event loop should stop.
    #[must_use]
    pub const fn should_quit(&self) -> bool {
        self.quit
    }
}
