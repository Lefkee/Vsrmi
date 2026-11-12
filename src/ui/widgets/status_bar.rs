//! # Status bar
//!
//! **Purpose:** show what file is open and what state it is in.
//!
//! **Responsibility:** one line at the bottom of the screen. The left side
//! carries the active mode, the file name and the dirty marker; the right side
//! carries the language, line ending, cursor position and scroll percentage.
//! Each mode gets its own accent colour so the current state is readable at a
//! glance.
//!
//! The right side is dropped rather than truncated when it would collide with
//! the file name on a narrow terminal.
//!
//! **Public API:** [`StatusBar`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::mode::Mode;
use crate::editor::cursor::Position;
use crate::editor::document::LineEnding;
use crate::theme::Theme;

/// The one-line information bar at the bottom of the text area.
pub struct StatusBar<'a> {
    /// Current mode, shown as a coloured label.
    pub mode: Mode,
    /// Name of the focused file, or `[No Name]` for an unnamed buffer.
    pub name: &'a str,
    /// Whether the buffer has unsaved changes.
    pub dirty: bool,
    /// Detected language, or `plain` when none applies.
    pub language: &'a str,
    /// Position of the primary cursor.
    pub position: Position,
    /// Total line count, used for the scroll percentage.
    pub line_count: usize,
    /// Line ending the document will be saved with.
    pub line_ending: LineEnding,
    /// Number of active cursors; only shown when there is more than one.
    pub cursor_count: usize,
    /// Colours.
    pub theme: &'a Theme,
}

impl StatusBar<'_> {
    /// Accent colour for the current mode.
    fn mode_style(&self) -> Style {
        let bg = match self.mode {
            Mode::Normal => Color::Rgb(0x89, 0xb4, 0xfa), // blue
            Mode::Insert => Color::Rgb(0xa6, 0xe3, 0xa1), // green
            Mode::Visual | Mode::VisualLine => Color::Rgb(0xcb, 0xa6, 0xf7), // mauve
            Mode::Command => Color::Rgb(0xfa, 0xb3, 0x87), // peach
            Mode::Search => Color::Rgb(0xf9, 0xe2, 0xaf), // yellow
            Mode::Tree => Color::Rgb(0x89, 0xdc, 0xeb),   // sky
        };
        Style::new()
            .fg(Color::Rgb(0x1e, 0x1e, 0x2e))
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Short label drawn for each mode.
    const fn mode_label(mode: Mode) -> &'static str {
        match mode {
            Mode::Normal => " ◆ NORMAL ",
            Mode::Insert => " ▶ INSERT ",
            Mode::Visual => " ▪ VISUAL ",
            Mode::VisualLine => " ▪ V-LINE ",
            Mode::Command => " : COMMAND",
            Mode::Search => " / SEARCH ",
            Mode::Tree => " ⊞ TREE   ",
        }
    }

    /// How far through the file the cursor is, as a percentage.
    fn scroll_percentage(&self) -> usize {
        let last = self.line_count.saturating_sub(1);
        (self.position.line * 100).checked_div(last).unwrap_or(100)
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }

        let base = self.theme.status;
        surface.set_style(area, base);

        // ── Left ──────────────────────────────────────────────────────────────
        // [mode label] [separator] [file name] [dirty mark] [cursor count]
        let mode_style = self.mode_style();
        let mut left = vec![
            Span::styled(Self::mode_label(self.mode), mode_style),
            // Block-style separator between the mode label and the file name.
            Span::styled(
                "▌",
                mode_style
                    .bg(base.bg.unwrap_or(Color::Reset))
                    .fg(mode_style.bg.unwrap_or(Color::Reset)),
            ),
            Span::styled(format!("  {} ", self.name), base),
        ];

        if self.dirty {
            left.push(Span::styled("● ", self.theme.status_dirty));
        }
        if self.cursor_count > 1 {
            left.push(Span::styled(
                format!(" ×{}", self.cursor_count),
                base.add_modifier(Modifier::ITALIC),
            ));
        }

        // ── Right ─────────────────────────────────────────────────────────────
        // [language] [line ending] [cursor count] [position] [scroll %]
        let pct = self.scroll_percentage();
        let scroll_icon = match pct {
            0 => "▁",
            1..=24 => "▃",
            25..=49 => "▅",
            50..=74 => "▆",
            75..=99 => "▇",
            _ => "█",
        };
        let mut segments = vec![
            self.language.to_string(),
            self.line_ending.label().to_string(),
        ];
        if self.cursor_count > 1 {
            segments.push(format!("{} cursors", self.cursor_count));
        }
        segments.push(format!(
            "Ln {} Col {}",
            self.position.line + 1,
            self.position.col + 1
        ));
        segments.push(format!("{scroll_icon}{pct}%"));

        let left = Line::from(left);
        let right =
            Line::from(Span::styled(format!(" {} ", segments.join("  ")), base)).right_aligned();

        // A right-aligned paragraph longer than the area draws over the left
        // side; omit it instead of letting it overwrite the file name.
        let show_right = left.width() + right.width() <= usize::from(area.width);
        left.render(area, surface);
        if show_right {
            right.render(area, surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(bar: StatusBar<'_>, width: u16) -> String {
        let backend = TestBackend::new(width, 1);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| frame.render_widget(bar, frame.area()))
            .expect("draw");
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    fn bar(theme: &Theme) -> StatusBar<'_> {
        StatusBar {
            mode: Mode::Normal,
            name: "main.rs",
            dirty: false,
            language: "rust",
            position: Position::new(0, 0),
            line_count: 10,
            line_ending: LineEnding::Lf,
            cursor_count: 1,
            theme,
        }
    }

    #[test]
    fn single_cursor_does_not_leave_an_empty_segment() {
        let theme = Theme::default();
        let line = render(bar(&theme), 80);
        assert!(line.contains("rust  LF  Ln 1 Col 1"));
        assert!(!line.contains("LF    Ln"));
    }

    #[test]
    fn narrow_bar_drops_the_right_side_instead_of_overwriting_the_name() {
        let theme = Theme::default();
        let line = render(bar(&theme), 24);
        assert!(line.contains("main.rs"));
        assert!(!line.contains("rust"));
    }
}
