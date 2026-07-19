//! # User interface
//!
//! **Purpose:** turn application state into a frame.
//!
//! **Responsibility:** split the terminal into regions and hand each one to a
//! widget. The UI layer reads the application state and writes pixels; it never
//! mutates editing state, with the single exception of scrolling the viewport,
//! which cannot be decided until the text area's size is known.
//!
//! **Public API:** [`draw`], [`Regions`].

pub mod text;
pub mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::App;
use crate::editor::selection::Range;
use crate::ui::widgets::{CommandBar, EditorView, StatusBar, Tab, TabBar, editor_view};

/// Where each part of the interface goes this frame.
#[derive(Debug, Clone, Copy)]
pub struct Regions {
    /// Tab strip, present only when more than one buffer is open.
    pub tabs: Option<Rect>,
    /// The text area, gutter included.
    pub editor: Rect,
    /// One-line status bar.
    pub status: Rect,
    /// One-line command bar, shared with transient messages.
    pub command: Rect,
}

impl Regions {
    /// Carve `area` into the editor's regions.
    ///
    /// The bars are fixed at one line each and are taken off the bottom first,
    /// so the editor absorbs every remaining row — and on a terminal too small
    /// to fit everything the flexible region shrinks rather than the fixed ones
    /// overlapping.
    #[must_use]
    pub fn split(area: Rect, show_tabs: bool) -> Self {
        let [tabs, body, status, command] = Layout::vertical([
            Constraint::Length(u16::from(show_tabs)),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        Self {
            tabs: show_tabs.then_some(tabs),
            editor: body,
            status,
            command,
        }
    }
}

/// Render one frame of the editor.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let show_tabs = app.config.show_tabs && app.buffers.len() > 1;
    let regions = Regions::split(frame.area(), show_tabs);
    // Page motions need the window height, which is only known here.
    app.viewport_height = regions.editor.height;

    // Scrolling has to happen before anything is drawn, and it is the only part
    // of rendering that mutates state. Destructuring keeps the borrow checker
    // happy about holding `config` and a buffer at the same time.
    {
        let App {
            buffers,
            active,
            config,
            ..
        } = &mut *app;
        editor_view::scroll_into_view(&mut buffers[*active], config, regions.editor);
    }

    let editor = EditorView {
        buffer: app.buffer(),
        theme: &app.theme,
        config: &app.config,
        selection: selection_range(app),
    };
    let editor_caret = editor.caret_position(regions.editor);
    frame.render_widget(editor, regions.editor);

    if let Some(area) = regions.tabs {
        let tabs: Vec<Tab<'_>> = app
            .buffers
            .iter()
            .map(|buffer| Tab {
                name: buffer.document.display_name(),
                dirty: buffer.document.is_dirty(),
            })
            .collect();
        frame.render_widget(
            TabBar {
                tabs: &tabs,
                active: app.active,
                theme: &app.theme,
            },
            area,
        );
    }

    frame.render_widget(status_bar(app), regions.status);

    let command_bar = CommandBar {
        command: app.mode.is_command().then_some(app.command_line.as_str()),
        status: &app.status,
        theme: &app.theme,
    };
    let command_caret = command_bar.caret_position(regions.command);
    frame.render_widget(command_bar, regions.command);

    if let Some(position) = command_caret.or(editor_caret) {
        frame.set_cursor_position(position);
    }
}

/// The span to paint as selected, which exists only in visual modes.
fn selection_range(app: &App) -> Option<Range> {
    let buffer = app.buffer();
    let cursor = buffer.cursor();
    match app.mode {
        crate::app::mode::Mode::Visual => Some(Range::of(&cursor, &buffer.document)),
        crate::app::mode::Mode::VisualLine => Some(Range::of_lines(&cursor, &buffer.document)),
        _ => None,
    }
}

/// Gather the values the status bar reports.
fn status_bar(app: &App) -> StatusBar<'_> {
    let buffer = app.buffer();
    StatusBar {
        mode: app.mode,
        name: buffer.document.display_name(),
        dirty: buffer.document.is_dirty(),
        language: "plain",
        position: buffer.cursor().head,
        line_count: buffer.document.len_lines(),
        line_ending: buffer.document.line_ending(),
        cursor_count: buffer.cursors().len(),
        theme: &app.theme,
    }
}
