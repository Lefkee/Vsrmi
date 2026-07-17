//! # Editor widget
//!
//! **Purpose:** draw the text.
//!
//! **Responsibility:** the line-number gutter, the visible slice of the
//! document, the current-line highlight and the selection. It writes cells
//! directly instead of building `Line`/`Span` values, because a full-screen
//! redraw touches every cell every frame and the intermediate allocations are
//! the one thing that shows up on large files.
//!
//! **Public API:** [`EditorView`], [`gutter_width`], [`scroll_into_view`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::config::Config;
use crate::editor::buffer::Buffer;
use crate::editor::selection::Range;
use crate::theme::Theme;
use crate::ui::text::DisplayLine;

/// Columns reserved for the line-number gutter, including its trailing space.
///
/// Returns `0` when line numbers are off, which makes the text area logic
/// uniform — there is no special case for "no gutter".
#[must_use]
pub fn gutter_width(config: &Config, line_count: usize) -> u16 {
    if !config.line_numbers {
        return 0;
    }
    let digits = line_count.max(1).ilog10() as u16 + 1;
    digits.max(3) + 2
}

/// Scroll the buffer's view so the primary caret is on screen.
///
/// Lives here rather than in `View` because the amount of usable width depends
/// on the gutter, and the caret's *display* column depends on tab expansion —
/// both of which are rendering concerns.
pub fn scroll_into_view(buffer: &mut Buffer, config: &Config, area: Rect) {
    let gutter = gutter_width(config, buffer.document.len_lines());
    let width = area.width.saturating_sub(gutter) as usize;
    let height = area.height as usize;

    let head = buffer.cursor().head;
    let column = if config.word_wrap {
        // Wrapped lines never scroll sideways.
        0
    } else {
        let line = DisplayLine::new(&buffer.document.line_string(head.line), config.tab_width);
        line.column_of(head.col)
    };
    buffer
        .view
        .scroll_to(head.line, column, height, width, config.scrolloff);
}

/// The text area and its gutter.
pub struct EditorView<'a> {
    /// Buffer to draw.
    pub buffer: &'a Buffer,
    /// Colours.
    pub theme: &'a Theme,
    /// Tab width, gutter and highlight settings.
    pub config: &'a Config,
    /// Span to paint with the selection style, if any.
    pub selection: Option<Range>,
}

impl EditorView<'_> {
    /// Screen position of the primary caret, or `None` when it is scrolled out
    /// of view.
    #[must_use]
    pub fn caret_position(&self, area: Rect) -> Option<(u16, u16)> {
        let head = self.buffer.cursor().head;
        let view = self.buffer.view;

        let row = head.line.checked_sub(view.top_line)?;
        if row >= area.height as usize {
            return None;
        }

        let line = DisplayLine::new(
            &self.buffer.document.line_string(head.line),
            self.config.tab_width,
        );
        let gutter = gutter_width(self.config, self.buffer.document.len_lines());
        let column = line.column_of(head.col).checked_sub(view.left_col)?;
        if column >= area.width.saturating_sub(gutter) as usize {
            return None;
        }

        Some((
            area.x + gutter + u16::try_from(column).ok()?,
            area.y + u16::try_from(row).ok()?,
        ))
    }

    /// Draw the line number for one row.
    fn render_gutter(&self, surface: &mut Surface, area: Rect, y: u16, line: usize, caret: usize) {
        let width = gutter_width(self.config, self.buffer.document.len_lines());
        if width == 0 {
            return;
        }
        let is_caret_line = line == caret;
        let number = if self.config.relative_line_numbers && !is_caret_line {
            line.abs_diff(caret)
        } else {
            line + 1
        };
        let style = if is_caret_line {
            self.theme.gutter_active
        } else {
            self.theme.gutter
        };

        // Right-align inside the gutter, leaving the final column as padding.
        let label = format!("{number:>width$} ", width = usize::from(width) - 1);
        for (offset, ch) in label.chars().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            if offset >= width {
                break;
            }
            if let Some(cell) = surface.cell_mut((area.x + offset, y)) {
                cell.set_char(ch).set_style(style);
            }
        }
    }

    /// Draw one line of text into the row at `y`.
    fn render_line(&self, surface: &mut Surface, area: Rect, y: u16, line: usize, caret: usize) {
        let gutter = gutter_width(self.config, self.buffer.document.len_lines());
        let x0 = area.x + gutter;
        let width = area.width.saturating_sub(gutter);

        let base = if line == caret && self.config.highlight_current_line {
            self.theme.text.patch(self.theme.cursor_line)
        } else {
            self.theme.text
        };

        let display = DisplayLine::new(
            &self.buffer.document.line_string(line),
            self.config.tab_width,
        );
        let line_start = self.buffer.document.line_start(line);
        let left = self.buffer.view.left_col;

        for column in 0..width {
            let Some(cell) = surface.cell_mut((x0 + column, y)) else {
                continue;
            };
            let index = left + usize::from(column);
            match display.cells.get(index) {
                Some(display_cell) => {
                    let style = self.style_for(base, line_start + display_cell.char_index);
                    match display_cell.glyph {
                        Some(glyph) => {
                            cell.set_char(glyph).set_style(style);
                        }
                        // Trailing half of a wide glyph: the glyph itself already
                        // covers this column, unless it was scrolled off the left
                        // edge, in which case draw a space to keep alignment.
                        None if column == 0 => {
                            cell.set_char(' ').set_style(style);
                        }
                        None => {
                            cell.set_symbol("").set_style(style);
                        }
                    }
                }
                // Past the end of the line: still paint the background so the
                // current-line highlight spans the full width.
                None => {
                    cell.set_char(' ').set_style(base);
                }
            }
        }
    }

    /// Combine the base style with the selection style when a character is
    /// selected.
    fn style_for(&self, base: Style, char_index: usize) -> Style {
        match self.selection {
            Some(range) if range.contains(char_index) => base.patch(self.theme.selection),
            _ => base,
        }
    }
}

impl Widget for EditorView<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }
        surface.set_style(area, self.theme.text);

        let document = &self.buffer.document;
        let caret = self.buffer.cursor().head.line;
        let top = self.buffer.view.top_line;

        for row in 0..area.height {
            let y = area.y + row;
            let line = top + usize::from(row);

            if line >= document.len_lines() {
                // Past the end of the document: a dim tilde, like vi.
                if let Some(cell) = surface.cell_mut((area.x, y)) {
                    cell.set_char('~').set_style(self.theme.gutter);
                }
                continue;
            }
            self.render_gutter(surface, area, y, line, caret);
            self.render_line(surface, area, y, line, caret);
        }
    }
}
