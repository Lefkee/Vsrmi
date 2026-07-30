//! # Built-in themes
//!
//! **Purpose:** ship two themes that work out of the box.
//!
//! **Responsibility:** hold the palettes and assemble them into [`Theme`]
//! values. Only this module contains literal colours — everywhere else asks the
//! theme for a slot.
//!
//! Both palettes use 24-bit RGB. Terminals without truecolor degrade these to
//! the nearest colour themselves, which looks better than hand-picking ANSI
//! indices that clash with the user's own palette.
//!
//! **Public API:** [`dark`], [`light`].
//!
//! The light palette is not the dark one inverted: readable light themes need
//! *darker*, more saturated accents, because a pastel that reads well on black
//! disappears on white.

use ratatui::style::{Color, Modifier, Style};

use super::{SyntaxStyles, Theme};

/// Deep dark theme — Catppuccin Mocha inspired.
///
/// Rich, deeply saturated colours on a near-black base so every syntax token
/// pops without burning the eyes after hours of editing.
#[must_use]
pub fn dark() -> Theme {
    // Base surface colours — four layers of depth.
    let bg       = Color::Rgb(0x1e, 0x1e, 0x2e); // Mocha base
    let bg_soft  = Color::Rgb(0x28, 0x28, 0x3d); // slightly lifted
    let bg_lift  = Color::Rgb(0x31, 0x32, 0x44); // overlay / status
    let bg_hl    = Color::Rgb(0x38, 0x3a, 0x4f); // cursor-line
    // Text colours.
    let fg       = Color::Rgb(0xcd, 0xd6, 0xf4); // Mocha text
    let fg_dim   = Color::Rgb(0x58, 0x5b, 0x70); // subtext / gutter
    // Accent colours (Mocha palette).
    let mauve    = Color::Rgb(0xcb, 0xa6, 0xf7); // keyword / macro
    let blue     = Color::Rgb(0x89, 0xb4, 0xfa); // function
    let sky      = Color::Rgb(0x89, 0xdc, 0xeb); // operator / type
    let green    = Color::Rgb(0xa6, 0xe3, 0xa1); // string / mode-insert
    let peach    = Color::Rgb(0xfa, 0xb3, 0x87); // number / constant / dirty
    let red      = Color::Rgb(0xf3, 0x8b, 0xa8); // error
    let yellow   = Color::Rgb(0xf9, 0xe2, 0xaf); // attribute / gutter-active
    let lavender = Color::Rgb(0xb4, 0xbe, 0xfe); // link

    Theme {
        name: "dark".to_string(),
        text:           Style::new().fg(fg).bg(bg),
        gutter:         Style::new().fg(fg_dim).bg(bg),
        gutter_active:  Style::new().fg(yellow).bg(bg_soft).add_modifier(Modifier::BOLD),
        cursor_line:    Style::new().bg(bg_hl),
        selection:      Style::new().bg(Color::Rgb(0x45, 0x47, 0x6a)),

        status:         Style::new().fg(fg).bg(bg_lift),
        status_mode:    Style::new().fg(bg).bg(green).add_modifier(Modifier::BOLD),
        status_dirty:   Style::new().fg(peach),

        command:        Style::new().fg(fg).bg(bg),
        command_error:  Style::new().fg(red).add_modifier(Modifier::BOLD),

        tab_active:     Style::new().fg(fg).bg(bg_lift).add_modifier(Modifier::BOLD),
        tab_inactive:   Style::new().fg(fg_dim).bg(bg),

        popup:          Style::new().fg(fg).bg(bg_lift),
        popup_border:   Style::new().fg(blue).bg(bg_lift),

        tree_directory: Style::new().fg(blue).add_modifier(Modifier::BOLD),
        tree_file:      Style::new().fg(fg),

        search:         Style::new().fg(bg).bg(yellow),
        search_active:  Style::new().fg(bg).bg(peach).add_modifier(Modifier::BOLD),

        syntax: SyntaxStyles {
            keyword:     Style::new().fg(mauve),
            type_name:   Style::new().fg(sky),
            function:    Style::new().fg(blue),
            string:      Style::new().fg(green),
            number:      Style::new().fg(peach),
            comment:     Style::new().fg(fg_dim).add_modifier(Modifier::ITALIC),
            constant:    Style::new().fg(peach),
            operator:    Style::new().fg(sky),
            punctuation: Style::new().fg(Color::Rgb(0x9a, 0x9e, 0xb5)),
            attribute:   Style::new().fg(yellow),
            macro_call:  Style::new().fg(mauve),
            heading:     Style::new().fg(blue).add_modifier(Modifier::BOLD),
            emphasis:    Style::new().add_modifier(Modifier::ITALIC),
            link:        Style::new().fg(lavender).add_modifier(Modifier::UNDERLINED),
        },
    }
}

/// Light theme — inspired by GitHub Light and Solarized.
#[must_use]
pub fn light() -> Theme {
    let bg       = Color::Rgb(0xf9, 0xf9, 0xf5);
    let bg_soft  = Color::Rgb(0xef, 0xee, 0xe8);
    let bg_lift  = Color::Rgb(0xe2, 0xe1, 0xda);
    let fg       = Color::Rgb(0x2b, 0x2d, 0x36);
    let fg_dim   = Color::Rgb(0x88, 0x90, 0x99);

    Theme {
        name: "light".to_string(),
        text:           Style::new().fg(fg).bg(bg),
        gutter:         Style::new().fg(fg_dim).bg(bg),
        gutter_active:  Style::new()
            .fg(Color::Rgb(0x8f, 0x62, 0x00))
            .bg(bg_soft)
            .add_modifier(Modifier::BOLD),
        cursor_line:    Style::new().bg(bg_soft),
        selection:      Style::new().bg(Color::Rgb(0xc7, 0xdb, 0xf0)),

        status:         Style::new().fg(fg).bg(bg_lift),
        status_mode:    Style::new()
            .fg(bg)
            .bg(Color::Rgb(0x2e, 0x7d, 0x32))
            .add_modifier(Modifier::BOLD),
        status_dirty:   Style::new().fg(Color::Rgb(0xb2, 0x5e, 0x09)),

        command:        Style::new().fg(fg).bg(bg),
        command_error:  Style::new()
            .fg(Color::Rgb(0xc0, 0x2c, 0x2c))
            .add_modifier(Modifier::BOLD),

        tab_active:     Style::new().fg(fg).bg(bg_lift).add_modifier(Modifier::BOLD),
        tab_inactive:   Style::new().fg(fg_dim).bg(bg),

        popup:          Style::new().fg(fg).bg(bg_lift),
        popup_border:   Style::new().fg(Color::Rgb(0x1e, 0x63, 0xa8)).bg(bg_lift),

        tree_directory: Style::new()
            .fg(Color::Rgb(0x1e, 0x63, 0xa8))
            .add_modifier(Modifier::BOLD),
        tree_file:      Style::new().fg(fg),

        search:         Style::new().fg(fg).bg(Color::Rgb(0xf5, 0xdd, 0x8f)),
        search_active:  Style::new()
            .fg(bg)
            .bg(Color::Rgb(0xd1, 0x71, 0x1c))
            .add_modifier(Modifier::BOLD),

        syntax: SyntaxStyles {
            keyword:     Style::new().fg(Color::Rgb(0x8b, 0x2c, 0x9e)),
            type_name:   Style::new().fg(Color::Rgb(0x0f, 0x6b, 0x63)),
            function:    Style::new().fg(Color::Rgb(0x1e, 0x63, 0xa8)),
            string:      Style::new().fg(Color::Rgb(0x2e, 0x6f, 0x1e)),
            number:      Style::new().fg(Color::Rgb(0xa6, 0x52, 0x00)),
            comment:     Style::new()
                .fg(Color::Rgb(0x8a, 0x92, 0x99))
                .add_modifier(Modifier::ITALIC),
            constant:    Style::new().fg(Color::Rgb(0xa6, 0x52, 0x00)),
            operator:    Style::new().fg(Color::Rgb(0x0d, 0x74, 0x89)),
            punctuation: Style::new().fg(Color::Rgb(0x55, 0x5d, 0x64)),
            attribute:   Style::new().fg(Color::Rgb(0x8f, 0x62, 0x00)),
            macro_call:  Style::new().fg(Color::Rgb(0x8b, 0x2c, 0x9e)),
            heading:     Style::new()
                .fg(Color::Rgb(0x1e, 0x63, 0xa8))
                .add_modifier(Modifier::BOLD),
            emphasis:    Style::new().add_modifier(Modifier::ITALIC),
            link:        Style::new()
                .fg(Color::Rgb(0x0d, 0x74, 0x89))
                .add_modifier(Modifier::UNDERLINED),
        },
    }
}
