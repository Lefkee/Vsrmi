//! # Action dispatch
//!
//! **Purpose:** carry out what the input layer decided.
//!
//! **Responsibility:** the single `match` from [`Action`] to an effect on the
//! application. Every feature the editor has passes through here exactly once,
//! which makes this file the place to look when asking "what does this key
//! actually do?".
//!
//! Dispatch owns the *policy* around edits — when to close an undo step, when a
//! mode change implies clamping the cursor, whether an operation applies to a
//! selection or to the current line. The mechanics live in the editor layer.
//!
//! **Public API:** [`apply`].

use anyhow::Result;

use super::mode::Mode;
use super::state::App;
use crate::editor::cursor::Motion;
use crate::editor::selection::Range;
use crate::input::Action;

/// Apply one action to the editor.
///
/// # Errors
/// Returns an error only for failures the user must see and that leave the
/// editor in a valid state, such as a failed save.
pub fn apply(app: &mut App, action: Action) -> Result<()> {
    // Any deliberate keystroke dismisses the previous message; keeping it around
    // makes it ambiguous which action it refers to.
    if !matches!(action, Action::None) {
        app.clear_status();
    }

    match action {
        Action::None => {}

        Action::EnterMode(mode) => enter_mode(app, mode),
        Action::Move(motion) => move_cursors(app, motion, false),
        Action::Extend(motion) => move_cursors(app, motion, true),
        Action::Scroll(delta) => {
            let last = app.buffer().document.last_line();
            app.buffer_mut().view.scroll_lines(delta, last);
        }
        Action::Page { down, half } => page(app, down, half),

        Action::Insert(ch) => {
            let (buffer, _) = app.buffer_and_config();
            buffer.insert_text(&ch.to_string());
        }
        Action::InsertNewline => {
            let (buffer, config) = app.buffer_and_config();
            buffer.insert_newline(config);
        }
        Action::InsertIndent => {
            let (buffer, config) = app.buffer_and_config();
            buffer.insert_indent(config);
        }
        Action::DeleteBackward => {
            let (buffer, config) = app.buffer_and_config();
            buffer.delete_backward(config);
        }
        Action::DeleteForward => app.buffer_mut().delete_forward(),
        Action::Delete => delete_target(app),

        Action::OpenLineBelow => open_line(app, true),
        Action::OpenLineAbove => open_line(app, false),
        Action::AppendAfter => {
            enter_mode(app, Mode::Insert);
            move_cursors(app, Motion::Right, false);
        }
        Action::AppendAtLineEnd => {
            enter_mode(app, Mode::Insert);
            move_cursors(app, Motion::LineEnd, false);
        }
        Action::InsertAtLineStart => {
            enter_mode(app, Mode::Insert);
            move_cursors(app, Motion::LineFirstNonBlank, false);
        }

        Action::Undo => {
            if !app.buffer_mut().undo() {
                app.info("already at the oldest change");
            }
        }
        Action::Redo => {
            if !app.buffer_mut().redo() {
                app.info("already at the newest change");
            }
        }

        Action::Yank => yank(app, false),
        Action::Cut => yank(app, true),
        Action::Paste => paste(app),

        Action::AddCursor { below } => add_cursor(app, below),
        Action::ClearCursors => {
            app.buffer_mut().clear_secondary_cursors();
            app.buffer_mut().collapse_selections();
        }

        Action::CycleBuffer { forward } => app.cycle_buffer(forward),
        Action::Save => save(app),
        Action::Quit => quit(app),

        Action::SearchStart { forward } => {
            let origin = origin_offset(app);
            app.search.begin(origin, forward);
            enter_mode(app, Mode::Search);
        }
        Action::SearchInput(ch) => {
            app.search.push(ch);
            seek_from_origin(app);
        }
        Action::SearchBackspace => {
            if app.search.query.is_empty() {
                return cancel_search(app);
            }
            app.search.pop();
            seek_from_origin(app);
        }
        Action::SearchSubmit => {
            if app.search.is_active() {
                let count = app.search.count(&app.buffer().document, MATCH_COUNT_CAP);
                app.info(format!(
                    "{count} match{}",
                    if count == 1 { "" } else { "es" }
                ));
            }
            enter_mode(app, Mode::Normal);
        }
        Action::SearchCancel => return cancel_search(app),
        Action::SearchRepeat { forward } => repeat_search(app, forward),

        Action::CommandInput(ch) => app.command_line.push(ch),
        Action::CommandBackspace => {
            app.command_line.pop();
            if app.command_line.is_empty() {
                // Backspacing past the `:` leaves command mode, as in vi.
                enter_mode(app, Mode::Normal);
            }
        }
        Action::CommandCancel => {
            app.command_line.clear();
            enter_mode(app, Mode::Normal);
        }
        Action::CommandSubmit => {
            let line = std::mem::take(&mut app.command_line);
            enter_mode(app, Mode::Normal);
            let _ = line;
        }
    }
    Ok(())
}

/// Upper bound on how many matches `:search` will count.
///
/// Counting is O(document); on a file with millions of hits the exact number is
/// not useful anyway, so it stops early and reports the cap.
const MATCH_COUNT_CAP: usize = 10_000;

/// Character offset of the primary caret.
fn origin_offset(app: &App) -> usize {
    let buffer = app.buffer();
    buffer.document.pos_to_char(buffer.cursor().head)
}

/// Jump to the first match at or after where the search started.
///
/// Searching from the origin rather than from the current match is what makes
/// deleting a character in the query walk *backwards* through the file instead
/// of leaving the caret stranded further down.
fn seek_from_origin(app: &mut App) {
    let origin = app.search.origin();
    let forward = app.search.forward;
    let Some(found) = app.search.find(&app.buffer().document, origin, forward) else {
        return;
    };
    let position = app.buffer().document.char_to_pos(found.start);
    app.buffer_mut().cursor_mut().move_to(position, false);
}

/// Leave search mode and put the caret back where it started.
fn cancel_search(app: &mut App) -> Result<()> {
    let origin = app.search.origin();
    let position = app.buffer().document.char_to_pos(origin);
    app.buffer_mut().cursor_mut().move_to(position, false);
    app.search.set_query(String::new());
    enter_mode(app, Mode::Normal);
    Ok(())
}

/// Jump to the next or previous match of the current query.
fn repeat_search(app: &mut App, forward: bool) {
    if !app.search.is_active() {
        app.info("no previous search");
        return;
    }
    let from = origin_offset(app);
    let Some(found) = app.search.find(&app.buffer().document, from, forward) else {
        app.error(format!("no match for {}", app.search.query));
        return;
    };
    let position = app.buffer().document.char_to_pos(found.start);
    app.buffer_mut().cursor_mut().move_to(position, false);
}

/// Switch modes, applying the invariants each mode requires.
fn enter_mode(app: &mut App, mode: Mode) {
    if app.mode == mode {
        return;
    }
    // A mode change always ends an undo step: undoing should return to the
    // state before this burst of typing, not the middle of it.
    app.buffer_mut().checkpoint();

    match mode {
        Mode::Normal => {
            app.buffer_mut().collapse_selections();
            // Normal mode's caret sits *on* a character, so a caret parked past
            // the end of a line in insert mode has to come back.
            app.buffer_mut().clamp_cursors(false);
        }
        Mode::Visual | Mode::VisualLine => app.buffer_mut().anchor_selections(),
        Mode::Command => app.command_line.clear(),
        Mode::Insert | Mode::Search => {}
    }
    app.mode = mode;
}

/// Move every cursor, extending the selection when asked.
fn move_cursors(app: &mut App, motion: Motion, extend: bool) {
    let allow_eol = app.mode.is_insert();
    let extend = extend || app.mode.is_visual();
    app.buffer_mut().checkpoint();
    app.buffer_mut().move_cursors(motion, extend, allow_eol);
}

/// Move a whole or half screen.
fn page(app: &mut App, down: bool, half: bool) {
    let height = usize::from(app.viewport_height).max(1);
    let distance = if half { height / 2 } else { height }.max(1);
    let motion = if down {
        Motion::Down(distance)
    } else {
        Motion::Up(distance)
    };
    move_cursors(app, motion, false);
}

/// The span an operator applies to: the selection in visual mode, the current
/// line otherwise.
fn target_range(app: &App) -> (Range, bool) {
    let buffer = app.buffer();
    let cursor = buffer.cursor();
    match app.mode {
        Mode::Visual => (Range::of(&cursor, &buffer.document), false),
        _ => (Range::of_lines(&cursor, &buffer.document), true),
    }
}

fn delete_target(app: &mut App) {
    let (range, _) = target_range(app);
    app.buffer_mut().delete_range(range);
    enter_mode(app, Mode::Normal);
}

fn yank(app: &mut App, cut: bool) {
    let (range, line_wise) = target_range(app);
    let text = app.buffer().document.slice_string(range.start, range.end);
    if text.is_empty() {
        return;
    }
    let lines = text.lines().count();
    app.clipboard.set(text, line_wise);

    if cut {
        app.buffer_mut().delete_range(range);
    }
    enter_mode(app, Mode::Normal);
    app.info(format!(
        "{} {lines} line{}",
        if cut { "cut" } else { "yanked" },
        if lines == 1 { "" } else { "s" }
    ));
}

fn paste(app: &mut App) {
    let text = app.clipboard.get();
    if text.is_empty() {
        app.info("clipboard is empty");
        return;
    }
    let line_wise = app.clipboard.is_line_wise();
    app.buffer_mut().paste(&text, line_wise);
    enter_mode(app, Mode::Normal);
}

/// Insert an empty line and start typing on it.
fn open_line(app: &mut App, below: bool) {
    enter_mode(app, Mode::Insert);
    // Splitting at the first non-blank character rather than at column zero is
    // what lets the new line inherit the current indentation.
    let motion = if below {
        Motion::LineEnd
    } else {
        Motion::LineFirstNonBlank
    };
    app.buffer_mut().move_cursors(motion, false, true);

    let (buffer, config) = app.buffer_and_config();
    buffer.insert_newline(config);
    if !below {
        // The split pushed the original text down; step back onto the blank
        // line that is now above it.
        buffer.move_cursors(Motion::Up(1), false, true);
    }
}

/// Duplicate the primary cursor onto the neighbouring line.
fn add_cursor(app: &mut App, below: bool) {
    let buffer = app.buffer_mut();
    let mut cursor = buffer.cursor();
    let last = buffer.document.last_line();

    let line = if below {
        (cursor.head.line + 1).min(last)
    } else {
        cursor.head.line.saturating_sub(1)
    };
    if line == cursor.head.line {
        return;
    }
    let position = buffer.document.clamp(
        crate::editor::cursor::Position::new(line, cursor.goal_col()),
        false,
    );
    cursor.move_to(position, false);
    buffer.add_cursor(cursor);
}

fn save(app: &mut App) {
    match app.buffer_mut().document.save() {
        Ok(()) => {
            let name = app.buffer().document.display_name().to_string();
            app.info(format!("wrote {name}"));
        }
        Err(error) => app.error(error.to_string()),
    }
}

fn quit(app: &mut App) {
    if app.has_unsaved_changes() {
        app.error("unsaved changes — use :q! to discard them");
    } else {
        app.quit();
    }
}
