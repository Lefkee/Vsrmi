//! # Application layer
//!
//! **Purpose:** own everything that is "the running editor" as opposed to "a
//! piece of text".
//!
//! **Responsibility:** holds the global state (open buffers, current mode,
//! pending command line, transient status message) and drives the event loop
//! that ties input, editing and rendering together.
//!
//! **Public API:** [`App`], [`run`].

pub mod dispatch;
pub mod mode;
pub mod state;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

pub use state::App;

use crate::input::Input;
use crate::renderer::Tui;
use crate::ui;

/// How long to block on input before waking up anyway.
///
/// The loop is otherwise fully event driven — this timeout only exists so that
/// background sources (file-system events, resize follow-ups) get serviced
/// promptly without spinning the CPU.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Run the editor until the user asks to quit.
///
/// # Errors
/// Returns an error if drawing a frame or reading an event fails.
pub fn run(app: &mut App, tui: &mut Tui) -> Result<()> {
    let mut input = Input::default();

    while !app.should_quit() {
        tui.set_cursor_shape(app.mode.uses_bar_cursor())?;
        tui.draw(|frame| ui::draw(frame, app))?;

        if !event::poll(POLL_INTERVAL)? {
            continue;
        }

        match event::read()? {
            // Windows reports both press and release; acting on both would
            // double every keystroke.
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let action = input.handle(key, app.mode);
                dispatch::apply(app, action)?;
            }
            Event::Paste(text) => {
                let (buffer, _) = app.buffer_and_config();
                buffer.insert_text(&text);
                buffer.checkpoint();
            }
            Event::Resize(_, _) => tui.clear()?,
            _ => {}
        }
    }
    Ok(())
}
