//! # Search box
//!
//! **Purpose:** the prompt shown while a search query is being typed.
//!
//! **Responsibility:** render `/query` or `?query` on the bottom line and report
//! where its caret goes. It shares the command bar's row — only one prompt can
//! be open at a time, so giving each its own line would waste a row forever to
//! save a branch.
//!
//! **Public API:** [`SearchBox`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// The incremental search prompt.
pub struct SearchBox<'a> {
    /// Query typed so far.
    pub query: &'a str,
    /// Direction, which decides the prompt character.
    pub forward: bool,
    /// Message from a regex that does not compile yet.
    pub error: Option<&'a str>,
    /// Colours.
    pub theme: &'a Theme,
}

impl SearchBox<'_> {
    /// The prompt character: `/` forwards, `?` backwards, as in vi.
    const fn sigil(&self) -> &'static str {
        if self.forward { "/" } else { "?" }
    }

    /// Screen position of the query caret.
    #[must_use]
    pub fn caret_position(&self, area: Rect) -> (u16, u16) {
        let column = u16::try_from(self.query.width()).unwrap_or(u16::MAX);
        (
            area.x
                .saturating_add(1 + column)
                .min(area.right().saturating_sub(1)),
            area.y,
        )
    }
}

impl Widget for SearchBox<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }
        surface.set_style(area, self.theme.command);

        let mut spans = vec![
            Span::styled(self.sigil(), self.theme.command),
            Span::styled(self.query, self.theme.command),
        ];
        if let Some(error) = self.error {
            spans.push(Span::styled(
                format!("  {error}"),
                self.theme.command.patch(self.theme.command_error),
            ));
        }
        Line::from(spans).render(area, surface);
    }
}
