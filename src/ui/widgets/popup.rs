//! # Popup
//!
//! **Purpose:** put a short message in front of everything else.
//!
//! **Responsibility:** centre a bordered box over the editor and wrap text
//! inside it. Used for anything that must be read before work continues — an
//! external-change warning, the key reference — as opposed to the status bar,
//! which is for things that can be missed.
//!
//! **Public API:** [`Popup`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap};

use crate::theme::Theme;

/// A centred modal message.
pub struct Popup<'a> {
    /// Title on the border.
    pub title: &'a str,
    /// Body text; blank lines separate paragraphs.
    pub body: &'a str,
    /// Hint shown at the bottom, such as which key dismisses it.
    pub hint: &'a str,
    /// Colours.
    pub theme: &'a Theme,
}

impl Popup<'_> {
    /// Centre a box of at most `width` × `height` inside `area`.
    ///
    /// Sized as a fraction of the screen with a hard cap, so it stays readable
    /// on a wide terminal and still fits on a narrow one.
    fn centred(area: Rect, width: u16, height: u16) -> Rect {
        let width = width.min(area.width.saturating_sub(2)).max(1);
        let height = height.min(area.height.saturating_sub(2)).max(1);
        Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + (area.height.saturating_sub(height)) / 2,
            width,
            height,
        }
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }
        let lines = u16::try_from(self.body.lines().count()).unwrap_or(u16::MAX);
        let popup = Self::centred(area, area.width * 3 / 4, lines + 4);

        // Without clearing, whatever the editor drew shows through the box.
        Clear.render(popup, surface);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.popup_border)
            .style(self.theme.popup)
            .title(Span::styled(
                format!(" {} ", self.title),
                self.theme.popup_border,
            ));
        let inner = block.inner(popup);
        block.render(popup, surface);

        let mut text: Vec<Line<'_>> = self
            .body
            .lines()
            .map(|line| Line::from(Span::styled(line, self.theme.popup)))
            .collect();
        if !self.hint.is_empty() {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(self.hint, self.theme.gutter)).right_aligned());
        }

        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .render(inner, surface);
    }
}
