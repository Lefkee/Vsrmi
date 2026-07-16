//! # Configuration
//!
//! **Purpose:** every user-tunable knob in one place.
//!
//! **Responsibility:** define the settings, their defaults, and how they are
//! read from TOML. Nothing else parses config files, and no module reaches for a
//! default of its own — if a value is configurable it lives here.
//!
//! Every field has a `Default`, and `#[serde(default)]` means a config file only
//! needs to mention what it changes. Unknown keys are rejected so a typo is
//! reported instead of silently ignored.
//!
//! **Public API:** [`Config`].

use serde::Deserialize;

/// User settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Theme name: `dark`, `light`, or the stem of a file in the themes
    /// directory.
    pub theme: String,
    /// Width of a tab stop in columns.
    pub tab_width: usize,
    /// Insert spaces instead of a literal tab character.
    pub expand_tabs: bool,
    /// Show the line-number gutter.
    pub line_numbers: bool,
    /// Number lines relative to the cursor, with the cursor's own line absolute.
    pub relative_line_numbers: bool,
    /// Copy the previous line's indentation onto new lines.
    pub auto_indent: bool,
    /// Wrap long lines instead of scrolling horizontally.
    pub word_wrap: bool,
    /// Highlight the line the cursor is on.
    pub highlight_current_line: bool,
    /// Lines of context to keep above and below the cursor while scrolling.
    pub scrolloff: usize,
    /// Enable syntax highlighting.
    pub syntax_highlighting: bool,
    /// Show the tab strip when more than one file is open.
    pub show_tabs: bool,
    /// Watch open files and warn when they change on disk.
    pub watch_files: bool,
    /// Strip trailing whitespace from every line on save.
    pub trim_trailing_whitespace: bool,
    /// Use the system clipboard for yank and paste. When `false`, an internal
    /// register is used instead.
    pub system_clipboard: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            tab_width: 4,
            expand_tabs: true,
            line_numbers: true,
            relative_line_numbers: false,
            auto_indent: true,
            word_wrap: false,
            highlight_current_line: true,
            scrolloff: 3,
            syntax_highlighting: true,
            show_tabs: true,
            watch_files: true,
            trim_trailing_whitespace: false,
            system_clipboard: true,
        }
    }
}

impl Config {
    /// Fix up values that would break rendering if taken literally.
    ///
    /// A `tab_width` of zero would make column arithmetic divide by zero, and an
    /// enormous `scrolloff` would pin the cursor to the middle of the screen.
    /// Clamping beats rejecting: the editor still starts.
    fn sanitise(&mut self) {
        self.tab_width = self.tab_width.clamp(1, 16);
        self.scrolloff = self.scrolloff.min(32);
    }
}
