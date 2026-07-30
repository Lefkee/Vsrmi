//! # Tab strip
//!
//! **Purpose:** show which files are open and which one has focus.
//!
//! **Responsibility:** one line listing the open buffers. When they do not all
//! fit, the strip scrolls so the active tab is always visible — truncating the
//! list from the left is what keeps the focused file on screen no matter how
//! many files are open.
//!
//! **Public API:** [`TabBar`], [`Tab`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// One entry in the strip.
#[derive(Debug, Clone, Copy)]
pub struct Tab<'a> {
    /// File name shown on the tab.
    pub name: &'a str,
    /// Whether the buffer has unsaved changes.
    pub dirty: bool,
}

/// The strip of open buffers.
pub struct TabBar<'a> {
    /// Open buffers, in order.
    pub tabs: &'a [Tab<'a>],
    /// Index of the focused tab.
    pub active: usize,
    /// Colours.
    pub theme: &'a Theme,
}

impl TabBar<'_> {
    /// Label for one tab: [indicator] name [dirty mark]
    fn label(tab: Tab<'_>, active: bool) -> String {
        // Active tab gets a filled triangle, inactive gets a small bullet.
        let icon = if active { " ▸ " } else { "  ▪ " };
        if tab.dirty {
            format!("{icon}{} ●  ", tab.name)
        } else {
            format!("{icon}{}  ", tab.name)
        }
    }

    /// First tab to draw so that the active one fits within `width`.
    ///
    /// Walks backwards from the active tab, adding earlier tabs while they fit.
    fn first_visible(&self, width: usize) -> usize {
        let mut used = 0;
        let mut first = self.active;
        for index in (0..=self.active).rev() {
            let needed = Self::label(self.tabs[index], index == self.active).width();
            if used + needed > width && index != self.active {
                break;
            }
            used += needed;
            first = index;
        }
        first
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() || self.tabs.is_empty() {
            return;
        }
        // Background for the whole strip.
        surface.set_style(area, self.theme.tab_inactive);

        let first = self.first_visible(area.width as usize);
        let mut spans: Vec<Span<'_>> = Vec::new();

        for (offset, tab) in self.tabs[first..].iter().enumerate() {
            let idx = first + offset;
            let is_active = idx == self.active;

            if is_active {
                let style = self.theme.tab_active;
                let label = Self::label(*tab, true);
                spans.push(Span::styled(label, style));
                // Block-style separator after active tab.
                let sep_style = self.theme.tab_active
                    .bg(self.theme.tab_inactive.bg.unwrap_or_default())
                    .fg(self.theme.tab_active.bg.unwrap_or_default())
                    .remove_modifier(Modifier::BOLD);
                spans.push(Span::styled("▌", sep_style));
            } else {
                spans.push(Span::styled(
                    Self::label(*tab, false),
                    self.theme.tab_inactive,
                ));
            }
        }

        Line::from(spans).render(area, surface);
    }
}
