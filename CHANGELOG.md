# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Autocomplete Snippets** — intelligent ghost-text autocomplete that predicts keywords, data types, and constants.
- **Local Word Completion** — the snippet engine also scans the current buffer (up to 1000 lines around the cursor) for words starting with your current prefix and suggests them.
- **Auto-closing Pairs** — typing brackets `()`, `[]`, `{}` or quotes `""`, `''` automatically inserts the closing pair, places the cursor in between, intelligently steps over existing closing characters, and deletes both when backspacing an empty pair.

## [0.1.0] - 2026-07-26

First release.

### Added

- **Text** — UTF-8 throughout, backed by a rope, so edit cost does not grow with
  file size. Tabs, double-width glyphs and mixed line endings are normalised at
  the edges.
- **Modal editing** — normal, insert, visual, visual-line, command and search
  modes, with vi-style motions and operators.
- **Multiple cursors** — modelled in the editing core rather than bolted on;
  `Alt+↑` / `Alt+↓` to add, `Esc` to collapse.
- **Undo** — inverse-operation history with consecutive typing merged into one
  step.
- **Search** — incremental, literal or regex, smart case, matches highlighted
  while typing, and `:%s/pattern/replacement/g`.
- **Syntax highlighting** — regex based, for Rust, C, C++, Zig, Python and
  Markdown, with per-line block-comment state cached so deep scrolling does not
  rescan the file.
- **Files** — multiple buffers with a tab strip, a lazily expanded file tree,
  atomic saves, and a watcher that reloads clean buffers and warns on dirty
  ones.
- **Theming** — dark and light built in, plus TOML themes that override only the
  slots they name.
- **Configuration** — TOML, with every key optional and unknown keys reported.

[Unreleased]: https://github.com/tuna4ll/termi/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tuna4ll/termi/releases/tag/v0.1.0
