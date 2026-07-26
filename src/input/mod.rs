//! # Input
//!
//! **Purpose:** turn key presses into intentions.
//!
//! **Responsibility:** own the [`Action`] vocabulary and the pending-key state
//! needed for multi-key sequences like `gg`. Nothing here touches a buffer —
//! translation and execution are separated so that keys can be remapped, or
//! actions replayed from a macro or a command, without duplicating any editing
//! logic.
//!
//! **Public API:** [`Action`], [`Input`].

pub mod keymap;

use crossterm::event::{KeyEvent, KeyModifiers};

use crate::app::mode::Mode;
use crate::editor::cursor::Motion;

/// Something the user asked the editor to do.
///
/// Actions are deliberately coarse — one per user-visible operation — so the
/// dispatcher reads like a list of features rather than a state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The key means nothing in this mode.
    None,
    /// Switch modes.
    EnterMode(Mode),
    /// Move every cursor.
    Move(Motion),
    /// Move every cursor, extending the selection.
    Extend(Motion),
    /// Scroll without moving the caret; negative is upwards.
    Scroll(isize),
    /// Move a page up or down, sized from the current window.
    Page { down: bool, half: bool },

    /// Insert a character at every cursor.
    Insert(char),
    /// Break the line.
    InsertNewline,
    /// Insert one indentation step.
    InsertIndent,
    /// Delete backwards.
    DeleteBackward,
    /// Delete forwards.
    DeleteForward,
    /// Delete the current selection or line.
    Delete,
    /// Open a new line below the current one and start inserting.
    OpenLineBelow,
    /// Open a new line above the current one and start inserting.
    OpenLineAbove,
    /// Enter insert mode after the caret.
    AppendAfter,
    /// Enter insert mode at the end of the line.
    AppendAtLineEnd,
    /// Enter insert mode at the first non-blank character.
    InsertAtLineStart,

    /// Undo one step.
    Undo,
    /// Redo one step.
    Redo,
    /// Copy the selection or line.
    Yank,
    /// Cut the selection or line.
    Cut,
    /// Paste the clipboard.
    Paste,

    /// Add a cursor on the line above or below.
    AddCursor { below: bool },
    /// Collapse back to a single cursor.
    ClearCursors,

    /// Focus the next or previous buffer.
    CycleBuffer { forward: bool },
    /// Write the active buffer.
    Save,
    /// Leave the editor.
    Quit,

    /// Show or hide the file tree panel.
    ToggleTree,
    /// Move the file tree selection by `delta` rows.
    TreeMove(isize),
    /// Open the selected file, or expand the selected directory.
    TreeActivate,

    /// Open the search prompt, searching forwards or backwards.
    SearchStart { forward: bool },
    /// Append a character to the search query.
    SearchInput(char),
    /// Remove the last character of the search query.
    SearchBackspace,
    /// Accept the search and keep the caret on the match.
    SearchSubmit,
    /// Abandon the search and go back to where it started.
    SearchCancel,
    /// Jump to the next or previous match of the last search.
    SearchRepeat { forward: bool },

    /// Append a character to the command line.
    CommandInput(char),
    /// Remove the last character of the command line.
    CommandBackspace,
    /// Run the command line.
    CommandSubmit,
    /// Abandon the command line.
    CommandCancel,
}

/// Key translation, including the state needed for multi-key sequences.
#[derive(Debug, Default)]
pub struct Input {
    /// First key of a pending sequence, such as the `g` of `gg`.
    ///
    /// Held here rather than in the application state because it is purely an
    /// input-layer concern and must be discarded whenever the mode changes.
    pending: Option<char>,
}

impl Input {
    /// Translate one key press in the context of `mode`.
    pub fn handle(&mut self, key: KeyEvent, mode: Mode) -> Action {
        if let Some(prefix) = self.pending.take() {
            return keymap::pending(prefix, key);
        }
        let action = match mode {
            Mode::Normal => keymap::normal(key, &mut self.pending),
            Mode::Insert => keymap::insert(key),
            Mode::Visual | Mode::VisualLine => keymap::visual(key, &mut self.pending),
            Mode::Command => keymap::command(key),
            Mode::Search => keymap::search(key),
            Mode::Tree => keymap::tree(key),
        };
        if matches!(action, Action::EnterMode(_)) {
            self.pending = None;
        }
        action
    }
}

/// Whether a key carries no modifier that changes its meaning.
///
/// Shift is ignored on purpose: crossterm already reports the shifted character,
/// so `Char('G')` arrives with `SHIFT` set and must still count as plain.
#[must_use]
pub(crate) fn is_plain(modifiers: KeyModifiers) -> bool {
    modifiers.difference(KeyModifiers::SHIFT).is_empty()
}

/// Whether a key is pressed with Control and nothing else.
#[must_use]
pub(crate) fn is_ctrl(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
}
