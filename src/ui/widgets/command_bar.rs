//! # Command bar
//!
//! **Purpose:** the bottom line — either the command being typed or the last
//! message.
//!
//! **Responsibility:** one line with two jobs that never happen at once. In
//! command mode it echoes `:` plus what has been typed and owns the caret;
//! otherwise it shows the most recent info or error message. Sharing the line is
//! what keeps the editor's chrome to two rows.
//!
//! **Public API:** [`CommandBar`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::app::state::Status;
use crate::theme::Theme;

/// The command line and message area.
pub struct CommandBar<'a> {
    /// Text typed after the leading `:`, or `None` when not in command mode.
    pub command: Option<&'a str>,
    /// Message to show when no command is being typed.
    pub status: &'a Status,
    /// Colours.
    pub theme: &'a Theme,
}

impl CommandBar<'_> {
    /// Screen position of the command-line caret, or `None` when the bar is
    /// showing a message instead.
    ///
    /// The caret is placed by display width rather than character count so it
    /// stays under the right glyph when a path contains wide characters.
    #[must_use]
    pub fn caret_position(&self, area: Rect) -> Option<(u16, u16)> {
        let command = self.command?;
        let column = u16::try_from(command.width()).unwrap_or(u16::MAX);
        Some((
            area.x
                .saturating_add(1 + column)
                .min(area.right().saturating_sub(1)),
            area.y,
        ))
    }
}

impl Widget for CommandBar<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }
        surface.set_style(area, self.theme.command);

        let line = match self.command {
            Some(command) => Line::from(vec![
                Span::styled(":", self.theme.command),
                Span::styled(command, self.theme.command),
            ]),
            None if self.status.text.is_empty() => return,
            None => {
                let style = if self.status.is_error {
                    self.theme.command.patch(self.theme.command_error)
                } else {
                    self.theme.command
                };
                Line::from(Span::styled(self.status.text.as_str(), style))
            }
        };
        line.render(area, surface);
    }
}
