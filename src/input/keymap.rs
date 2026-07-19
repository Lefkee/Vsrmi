//! # Keymaps
//!
//! **Purpose:** the actual key bindings.
//!
//! **Responsibility:** one function per mode, each a flat `match` from key to
//! [`Action`]. A table rather than nested conditionals means the whole binding
//! set for a mode can be read — and audited for conflicts — in one screen.
//!
//! The bindings follow vi where vi is unambiguous, and Helix where vi is
//! awkward; the editing keys (`Ctrl+S`, `Ctrl+Q`, arrows, Home/End) also work
//! the way a modeless editor's do, so the editor is usable before the modal
//! bindings are learned.
//!
//! **Public API:** [`normal`], [`insert`], [`visual`], [`command`],
//! [`pending`].

use crossterm::event::{KeyCode, KeyEvent};

use super::{Action, is_ctrl, is_plain};
use crate::app::mode::Mode;
use crate::editor::cursor::Motion;

/// Bindings shared by every mode: window-level keys that must always work.
fn universal(key: KeyEvent) -> Option<Action> {
    if !is_ctrl(key.modifiers) {
        return None;
    }
    Some(match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('s') => Action::Save,
        KeyCode::Char('n') => Action::CycleBuffer { forward: true },
        KeyCode::Char('p') => Action::CycleBuffer { forward: false },
        KeyCode::Char('z') => Action::Undo,
        KeyCode::Char('y' | 'r') => Action::Redo,
        KeyCode::Char('d') => Action::Page {
            down: true,
            half: true,
        },
        KeyCode::Char('u') => Action::Page {
            down: false,
            half: true,
        },
        KeyCode::Char('e') => Action::Scroll(1),
        KeyCode::Char('b') => Action::Scroll(-1),
        _ => return None,
    })
}

/// Motions that mean the same thing in normal and visual mode.
fn motion(key: KeyEvent) -> Option<Motion> {
    if !is_plain(key.modifiers) {
        return None;
    }
    Some(match key.code {
        KeyCode::Char('h') | KeyCode::Left => Motion::Left,
        KeyCode::Char('l') | KeyCode::Right => Motion::Right,
        KeyCode::Char('k') | KeyCode::Up => Motion::Up(1),
        KeyCode::Char('j') | KeyCode::Down => Motion::Down(1),
        KeyCode::Char('w') => Motion::WordForward,
        KeyCode::Char('b') => Motion::WordBackward,
        KeyCode::Char('e') => Motion::WordEnd,
        KeyCode::Char('0') | KeyCode::Home => Motion::LineStart,
        KeyCode::Char('^') => Motion::LineFirstNonBlank,
        KeyCode::Char('$') | KeyCode::End => Motion::LineEnd,
        KeyCode::Char('G') => Motion::DocEnd,
        _ => return None,
    })
}

/// Normal mode: keys are commands.
///
/// `pending` is set when a key only makes sense as the first half of a sequence.
pub fn normal(key: KeyEvent, pending: &mut Option<char>) -> Action {
    if let Some(action) = universal(key) {
        return action;
    }
    if let Some(motion) = motion(key) {
        return Action::Move(motion);
    }
    if !is_plain(key.modifiers) {
        return Action::None;
    }
    match key.code {
        KeyCode::Char('i') => Action::EnterMode(Mode::Insert),
        KeyCode::Char('I') => Action::InsertAtLineStart,
        KeyCode::Char('a') => Action::AppendAfter,
        KeyCode::Char('A') => Action::AppendAtLineEnd,
        KeyCode::Char('o') => Action::OpenLineBelow,
        KeyCode::Char('O') => Action::OpenLineAbove,
        KeyCode::Char('v') => Action::EnterMode(Mode::Visual),
        KeyCode::Char('V') => Action::EnterMode(Mode::VisualLine),
        KeyCode::Char(':') => Action::EnterMode(Mode::Command),

        KeyCode::Char('x') | KeyCode::Delete => Action::DeleteForward,
        KeyCode::Backspace => Action::DeleteBackward,
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Char('p') => Action::Paste,

        // Sequences: the second key decides what happens.
        KeyCode::Char(prefix @ ('g' | 'd' | 'y')) => {
            *pending = Some(prefix);
            Action::None
        }

        KeyCode::PageDown => Action::Page {
            down: true,
            half: false,
        },
        KeyCode::PageUp => Action::Page {
            down: false,
            half: false,
        },
        KeyCode::Esc => Action::ClearCursors,
        _ => Action::None,
    }
}

/// The second key of a sequence started in normal or visual mode.
pub fn pending(prefix: char, key: KeyEvent) -> Action {
    match (prefix, key.code) {
        ('g', KeyCode::Char('g')) => Action::Move(Motion::DocStart),
        ('g', KeyCode::Char('e')) => Action::Move(Motion::DocEnd),
        ('g', KeyCode::Char('h')) => Action::Move(Motion::LineStart),
        ('g', KeyCode::Char('l')) => Action::Move(Motion::LineEnd),
        ('g', KeyCode::Char('s')) => Action::Move(Motion::LineFirstNonBlank),
        ('d', KeyCode::Char('d')) => Action::Delete,
        ('y', KeyCode::Char('y')) => Action::Yank,
        _ => Action::None,
    }
}

/// Insert mode: printable keys become text.
pub fn insert(key: KeyEvent) -> Action {
    if let Some(action) = universal(key) {
        return action;
    }
    match key.code {
        KeyCode::Esc => Action::EnterMode(Mode::Normal),
        KeyCode::Enter => Action::InsertNewline,
        KeyCode::Tab => Action::InsertIndent,
        KeyCode::Backspace => Action::DeleteBackward,
        KeyCode::Delete => Action::DeleteForward,

        KeyCode::Left => Action::Move(Motion::Left),
        KeyCode::Right => Action::Move(Motion::Right),
        KeyCode::Up => Action::Move(Motion::Up(1)),
        KeyCode::Down => Action::Move(Motion::Down(1)),
        KeyCode::Home => Action::Move(Motion::LineStart),
        KeyCode::End => Action::Move(Motion::LineEnd),
        KeyCode::PageUp => Action::Page {
            down: false,
            half: false,
        },
        KeyCode::PageDown => Action::Page {
            down: true,
            half: false,
        },

        // Anything else printable is text. Control combinations were already
        // handled above, so reaching here with a modifier means Shift or AltGr,
        // both of which are part of the character crossterm reports.
        KeyCode::Char(ch) if is_plain(key.modifiers) => Action::Insert(ch),
        _ => Action::None,
    }
}

/// Visual mode: motions extend the selection.
pub fn visual(key: KeyEvent, pending: &mut Option<char>) -> Action {
    if let Some(action) = universal(key) {
        return action;
    }
    if let Some(motion) = motion(key) {
        return Action::Extend(motion);
    }
    if !is_plain(key.modifiers) {
        return Action::None;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('v') => Action::EnterMode(Mode::Normal),
        KeyCode::Char('V') => Action::EnterMode(Mode::VisualLine),
        KeyCode::Char(':') => Action::EnterMode(Mode::Command),
        KeyCode::Char('d' | 'x') | KeyCode::Delete => Action::Delete,
        KeyCode::Char('y') => Action::Yank,
        KeyCode::Char('c') => Action::Cut,
        KeyCode::Char('p') => Action::Paste,
        KeyCode::Char('i') => Action::EnterMode(Mode::Insert),
        KeyCode::Char('g') => {
            *pending = Some('g');
            Action::None
        }
        KeyCode::PageDown => Action::Page {
            down: true,
            half: false,
        },
        KeyCode::PageUp => Action::Page {
            down: false,
            half: false,
        },
        _ => Action::None,
    }
}

/// Command mode: the command bar owns the keyboard.
pub fn command(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::CommandCancel,
        KeyCode::Enter => Action::CommandSubmit,
        KeyCode::Backspace => Action::CommandBackspace,
        KeyCode::Char(ch) if is_plain(key.modifiers) => Action::CommandInput(ch),
        _ => Action::None,
    }
}
