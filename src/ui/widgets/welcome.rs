//! # Welcome panel
//!
//! **Purpose:** tell someone who opened an empty editor where to start.
//!
//! **Responsibility:** draw the wordmark, version and a short list of useful
//! keys over the editor area. The widget is purely decorative: it never
//! consumes input and disappears as soon as the buffer has text.
//!
//! **Public API:** [`Welcome`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};

use crate::theme::Theme;

const FULL_MIN_WIDTH: u16 = 60;
const FULL_MIN_HEIGHT: u16 = 15;
const PANEL_WIDTH: u16 = 56;
const PANEL_HEIGHT: u16 = 13;
const COMPACT_MIN_WIDTH: u16 = 20;

/// Welcome panel shown on a bare start.
pub struct Welcome<'a> {
    /// Version shown in the panel.
    pub version: &'a str,
    /// Colours.
    pub theme: &'a Theme,
}

impl Welcome<'_> {
    /// Build the eleven lines of the full panel.
    fn full_lines(&self) -> Vec<Line<'static>> {
        let shortcut = |key: &str, label: &str| {
            Line::from(vec![
                Span::styled(format!("{key:<8}"), self.theme.syntax.function),
                Span::styled(label.to_string(), self.theme.text),
            ])
        };

        vec![
            Line::default(),
            Line::from(vec![
                Span::styled(
                    "vsrmi".to_string(),
                    self.theme.syntax.keyword.add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(format!("v{}", self.version), self.theme.gutter),
            ]),
            Line::styled("a modal terminal code editor", self.theme.gutter),
            Line::default(),
            shortcut("i", "insert mode"),
            shortcut(":help", "every key"),
            shortcut("Ctrl+B", "file tree"),
            shortcut("Ctrl+S", "save"),
            Line::default(),
            Line::styled("Press i to start · :e <file> to open", self.theme.gutter),
            Line::default(),
        ]
    }

    /// Build the two-line fallback used below the full panel's threshold.
    fn compact_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::styled(format!("vsrmi v{}", self.version), self.theme.gutter),
            Line::styled("i to start · :help · Ctrl+B", self.theme.gutter),
        ]
    }

    /// Centre a `width` × `height` rectangle inside `area`.
    fn centred(area: Rect, width: u16, height: u16) -> Rect {
        Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        }
    }
}

impl Widget for Welcome<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        // Without clearing, the blank document's line-number gutter shows
        // through the panel.
        Clear.render(area, surface);
        surface.set_style(area, self.theme.text);

        if area.width >= FULL_MIN_WIDTH && area.height >= FULL_MIN_HEIGHT {
            let panel = Self::centred(area, PANEL_WIDTH, PANEL_HEIGHT);
            let block = Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(self.theme.popup_border)
                .style(self.theme.text);
            let inner = block.inner(panel);
            block.render(panel, surface);
            Paragraph::new(self.full_lines())
                .alignment(Alignment::Center)
                .render(inner, surface);
        } else if area.width >= COMPACT_MIN_WIDTH && area.height >= 2 {
            let compact = Self::centred(area, area.width, 2);
            Paragraph::new(self.compact_lines())
                .alignment(Alignment::Center)
                .render(compact, surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let theme = Theme::default();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    Welcome {
                        version: "0.1.0",
                        theme: &theme,
                    },
                    frame.area(),
                );
            })
            .expect("draw");
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn full_panel_shows_brand_version_and_shortcuts() {
        let screen = render(80, 24);
        for needle in ["vsrmi", "0.1.0", "Ctrl+B", ":e <file>"] {
            assert!(screen.contains(needle), "missing {needle}");
        }
    }

    #[test]
    fn compact_panel_fits_narrow_terminals() {
        let screen = render(40, 6);
        assert!(screen.contains("i to start"));
        assert!(screen.contains(":help"));
        assert!(!screen.contains("insert mode"));
    }

    #[test]
    fn tiny_terminals_do_not_panic() {
        let _ = render(19, 2);
        let _ = render(1, 1);
    }
}
