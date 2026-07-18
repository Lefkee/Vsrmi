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
    /// Label for one tab, including padding and the unsaved marker.
    fn label(tab: Tab<'_>) -> String {
        if tab.dirty {
            format!(" {} ● ", tab.name)
        } else {
            format!(" {} ", tab.name)
        }
    }

    /// First tab to draw so that the active one fits within `width`.
    ///
    /// Walks backwards from the active tab, adding earlier tabs while they fit.
    fn first_visible(&self, width: usize) -> usize {
        let mut used = 0;
        let mut first = self.active;
        for index in (0..=self.active).rev() {
            let needed = Self::label(self.tabs[index]).width();
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
        surface.set_style(area, self.theme.tab_inactive);

        let first = self.first_visible(area.width as usize);
        let spans: Vec<Span<'_>> = self.tabs[first..]
            .iter()
            .enumerate()
            .map(|(offset, tab)| {
                let style = if first + offset == self.active {
                    self.theme.tab_active
                } else {
                    self.theme.tab_inactive
                };
                Span::styled(Self::label(*tab), style)
            })
            .collect();

        Line::from(spans).render(area, surface);
    }
}
