# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-12

First release.

### Added

- **Welcome screen** — the bare start shows the version and the few shortcuts
  worth memorising; it disappears as soon as the buffer has text.
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
- **Syntax highlighting** — regex based, for Rust, C, C++, Zig, Python,
  Markdown, TOML, JSON, YAML and Shell, with per-line block-comment state cached
  so deep scrolling does not rescan the file.
- **Files** — multiple buffers with a tab strip, a lazily expanded file tree,
  atomic saves, and a watcher that reloads clean buffers and warns on dirty
  ones.
- **Theming** — dark and light built in, plus TOML themes that override only the
  slots they name.
- **Configuration** — TOML, with every key optional and unknown keys reported.
- **Autocomplete snippets** — ghost-text completion for keywords, types,
  constants and words already in the buffer.
- **Auto-closing pairs** — brackets and quotes insert their partner, step over an
  existing closer, and delete both when backspacing an empty pair.

### Changed

- The status bar keeps its right-hand information intact on narrow terminals
  instead of drawing over the file name.
- The help popup shows the current version, multi-cursor/undo keys and the
  supported languages.
- Dependencies refreshed; `dirs` 7 and `arboard` 3.7 replace `dirs` 6 and
  `arboard` 3.6.

[Unreleased]: https://github.com/Lefkee/Vsrmi/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Lefkee/Vsrmi/releases/tag/v0.1.0
