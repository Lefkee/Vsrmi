//! # File tree
//!
//! **Purpose:** the side panel listing the project's files.
//!
//! **Responsibility:** draw a flattened [`Tree`](crate::filesystem::tree::Tree)
//! with indentation, folder markers, file-type icons and a selection highlight,
//! scrolled so the selected row stays visible. The tree itself decides what is
//! expanded; this widget only draws it.
//!
//! **Public API:** [`FileTree`].

use ratatui::buffer::Buffer as Surface;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::filesystem::tree::Entry;
use crate::theme::Theme;

/// The file browser panel.
pub struct FileTree<'a> {
    /// Rows to draw, already flattened.
    pub entries: &'a [Entry],
    /// Index of the highlighted row.
    pub selected: usize,
    /// Whether the panel has keyboard focus.
    pub focused: bool,
    /// Title shown on the border, usually the root directory's name.
    pub title: &'a str,
    /// Colours.
    pub theme: &'a Theme,
}

impl FileTree<'_> {
    /// First row to draw so `selected` is visible in a panel `height` tall.
    fn scroll_offset(&self, height: usize) -> usize {
        if height == 0 || self.selected < height {
            return 0;
        }
        self.selected - height + 1
    }

    /// Icon, indentation and name for one entry.
    ///
    /// Folders get arrow icons that reflect their open/closed state. Files get
    /// a small icon chosen from a compact set keyed on the extension.
    fn label(entry: &Entry) -> String {
        let indent = "  ".repeat(entry.depth);
        if entry.is_dir {
            let arrow = if entry.is_open { "▾ " } else { "▸ " };
            format!("{indent} {arrow}{}", entry.name)
        } else {
            let icon = file_icon(&entry.name);
            format!("{indent}  {icon} {}", entry.name)
        }
    }
}

/// Pick a short text label for a filename extension.
///
/// Falls back to a generic marker for unrecognised extensions.
fn file_icon(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs"                               => "[rs]",
        "py"                               => "[py]",
        "js" | "mjs"                       => "[js]",
        "ts"                               => "[ts]",
        "html" | "htm"                     => "[ht]",
        "css"                              => "[cs]",
        "json"                             => "[js]",
        "toml" | "yaml" | "yml"            => "[cf]",
        "md" | "markdown"                  => "[md]",
        "c" | "h"                          => "[c] ",
        "cpp" | "cc" | "cxx" | "hpp"       => "[c+]",
        "zig"                              => "[zg]",
        "go"                               => "[go]",
        "sh" | "bash" | "zsh"             => "[sh]",
        "txt"                              => "[tx]",
        "lock"                             => "[lk]",
        "git" | "gitignore"                => "[gi]",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "[im]",
        _                                  => "[  ]",
    }
}

impl Widget for FileTree<'_> {
    fn render(self, area: Rect, surface: &mut Surface) {
        if area.is_empty() {
            return;
        }

        let border_style = if self.focused {
            self.theme.popup_border
        } else {
            self.theme.gutter
        };

        // Show root directory name in the title.
        let title_text = format!(" ▤ {}", self.title);
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_type(BorderType::Plain)
            .border_style(border_style)
            .title(Span::styled(title_text, self.theme.tree_directory));

        let inner = block.inner(area);
        surface.set_style(area, self.theme.text);
        block.render(area, surface);

        let height = inner.height as usize;
        let offset = self.scroll_offset(height);

        for (row, entry) in self.entries.iter().skip(offset).take(height).enumerate() {
            let index = offset + row;
            let mut style = if entry.is_dir {
                self.theme.tree_directory
            } else {
                self.theme.tree_file
            };
            if index == self.selected {
                style = style.patch(if self.focused {
                    self.theme.selection
                } else {
                    self.theme.cursor_line
                });
            }

            let y = inner.y + u16::try_from(row).unwrap_or(u16::MAX);
            let line = Rect { y, height: 1, ..inner };

            // Paint the full row before drawing the text so the selection
            // highlight fills the entire width.
            surface.set_style(line, style);
            Line::from(Span::styled(Self::label(entry), style)).render(line, surface);
        }
    }
}
