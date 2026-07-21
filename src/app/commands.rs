//! # Command execution
//!
//! **Purpose:** carry out a parsed `:` command.
//!
//! **Responsibility:** the effect half of [`crate::editor::command`]. Every arm
//! ends by putting a message in the status area, because a command line that
//! silently does nothing is indistinguishable from one that failed.
//!
//! **Public API:** [`run`].

use std::path::PathBuf;

use super::mode::Mode;
use super::state::App;
use crate::config;
use crate::editor::command::{Command, parser};
use crate::editor::cursor::Motion;
use crate::theme::Theme;

/// Parse and execute a command line, without the leading `:`.
pub fn run(app: &mut App, line: &str) {
    match parser::parse(line) {
        Ok(command) => execute(app, command),
        Err(message) => app.error(message),
    }
}

fn execute(app: &mut App, command: Command) {
    match command {
        Command::Write(path) => write(app, path),
        Command::Quit { force } => quit(app, force),
        Command::WriteQuit { force } => {
            write(app, None);
            if !app.status.is_error {
                quit(app, force);
            }
        }
        Command::Edit { path, force } => edit(app, path, force),
        Command::Reload => reload(app),
        Command::GotoLine(line) => {
            app.buffer_mut()
                .move_cursors(Motion::ToLine(line), false, false);
        }
        Command::Set { key, value } => set_option(app, &key, &value),
        Command::Theme(name) => set_theme(app, &name),
        Command::CycleBuffer { forward } => app.cycle_buffer(forward),
        Command::Substitute {
            pattern,
            replacement,
            all,
            whole_file,
        } => substitute(app, &pattern, &replacement, all, whole_file),
        Command::Help => app.info(
            ":w :q :wq :e :bn :bp :set :theme :s/a/b/g  —  / search, n next, u undo, Ctrl+Q quit",
        ),
    }
}

fn write(app: &mut App, path: Option<PathBuf>) {
    let result = match path {
        Some(path) => app.buffer_mut().document.save_as(path),
        None => app.buffer_mut().document.save(),
    };
    match result {
        Ok(()) => {
            let name = app.buffer().document.display_name().to_string();
            let lines = app.buffer().document.len_lines();
            app.info(format!("wrote {name} — {lines} lines"));
        }
        Err(error) => app.error(error.to_string()),
    }
}

/// `:q` closes the buffer; closing the last one leaves the editor.
fn quit(app: &mut App, force: bool) {
    if !force && app.buffer().document.is_dirty() {
        app.error("unsaved changes — use :q! to discard them");
        return;
    }
    if app.buffers.len() == 1 {
        app.quit();
    } else {
        app.close_active();
    }
}

fn edit(app: &mut App, path: PathBuf, force: bool) {
    if !force && app.buffer().document.is_dirty() {
        app.error("unsaved changes — use :e! to discard them");
        return;
    }
    match app.open(path) {
        Ok(()) => {
            let name = app.buffer().document.display_name().to_string();
            app.info(format!("opened {name}"));
        }
        Err(error) => app.error(error.to_string()),
    }
}

fn reload(app: &mut App) {
    match app.buffer_mut().document.reload() {
        Ok(()) => {
            // The file may have shrunk, so no cursor can be trusted afterwards.
            app.buffer_mut().clear_secondary_cursors();
            app.buffer_mut().clamp_cursors(false);
            app.info("reloaded from disk");
        }
        Err(error) => app.error(error.to_string()),
    }
}

fn set_theme(app: &mut App, name: &str) {
    let (theme, error) = Theme::load(name, &config::themes_dir());
    app.theme = theme;
    app.config.theme = name.to_string();
    match error {
        Some(message) => app.error(message),
        None => app.info(format!("theme: {}", app.theme.name)),
    }
}

/// Change one setting for the current session.
///
/// Names accept the vi spellings as well as the config-file ones, so muscle
/// memory from either direction works.
fn set_option(app: &mut App, key: &str, value: &str) {
    let boolean = || matches!(value, "true" | "on" | "yes" | "1");
    let number = |fallback: usize| value.parse::<usize>().unwrap_or(fallback);

    match key {
        "number" | "nu" | "line_numbers" => app.config.line_numbers = boolean(),
        "relativenumber" | "rnu" | "relative_line_numbers" => {
            app.config.relative_line_numbers = boolean();
        }
        "wrap" | "word_wrap" => app.config.word_wrap = boolean(),
        "autoindent" | "ai" | "auto_indent" => app.config.auto_indent = boolean(),
        "expandtab" | "et" | "expand_tabs" => app.config.expand_tabs = boolean(),
        "cursorline" | "cul" | "highlight_current_line" => {
            app.config.highlight_current_line = boolean();
        }
        "syntax" | "syntax_highlighting" => app.config.syntax_highlighting = boolean(),
        "tabs" | "show_tabs" => app.config.show_tabs = boolean(),
        "tabstop" | "ts" | "tab_width" => {
            app.config.tab_width = number(app.config.tab_width).clamp(1, 16);
        }
        "scrolloff" | "so" => app.config.scrolloff = number(app.config.scrolloff).min(32),
        "ignorecase" | "ic" => app.search.case_sensitive = Some(!boolean()),
        "regex" => app.search.regex = boolean(),
        "theme" => return set_theme(app, value),
        other => return app.error(format!("unknown option: {other}")),
    }
    app.info(format!("{key} = {value}"));
}

fn substitute(app: &mut App, pattern: &str, replacement: &str, all: bool, whole_file: bool) {
    let previous = std::mem::take(&mut app.search.query);
    app.search.set_query(pattern.to_string());

    if let Some(error) = app.search.error() {
        let error = error.to_string();
        app.search.set_query(previous);
        return app.error(error);
    }

    let lines = if whole_file {
        0..app.buffer().document.len_lines()
    } else {
        let line = app.buffer().cursor().head.line;
        line..line + 1
    };

    let App {
        buffers,
        active,
        search,
        ..
    } = app;
    let replaced = search.replace_in(&mut buffers[*active].document, lines, replacement, all);

    // Substitution rewrites whole lines, so the caret may now be past the end.
    app.buffer_mut().clamp_cursors(false);
    // Rewriting lines wholesale cannot be expressed as one coalesced edit, so
    // close the undo step to keep the next keystroke separate.
    app.buffer_mut().checkpoint();

    if replaced == 0 {
        app.error(format!("no match for {pattern}"));
    } else {
        app.info(format!("replaced {replaced} occurrence(s)"));
    }
    app.mode = Mode::Normal;
}
