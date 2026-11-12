# Vsrmi 0.1.0 Comprehensive Renewal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a visibly polished, working Vsrmi 0.1.0: welcome screen, four new languages, fixed status bar/help, refreshed dependencies, metadata, README and CI.

**Architecture:** New UI stays inside the existing widget layer: a `Welcome` widget renders over the editor region only when the active document is an untouched scratch buffer. New languages are data-only definitions in `src/syntax/languages/`; the regex engine is untouched. Status bar change is a width-aware render pass. Dependency, version and documentation work is isolated to metadata files.

**Tech Stack:** Rust 2024, ratatui 0.30, crossterm 0.29, ropey 1.6, regex 1.13, toml 1.2, serde 1, dirs 7 (fallback 6), arboard 3.7, notify 8.

**Spec:** `docs/superpowers/specs/2026-11-12-vsrmi-renewal-design.md`

## Global Constraints

- Package name stays `vsrmi`; version becomes exactly `0.1.0`.
- `rust-version = "1.88"`; `unsafe_code = "forbid"`.
- Keep `notify = "8"` and `ropey = "1.6"`; `notify` 9 is RC, `ropey` 2 is beta.
- `dirs = "7"` only if it compiles with Rust 1.88; otherwise keep `dirs = "6"` and record the deviation in `CHANGELOG.md`.
- No new theme slots: `Welcome` uses `text`, `gutter`, `syntax.keyword`, `syntax.function`, `popup_border`.
- No command palette, recent-files persistence or session restore.
- No `v0.1.0` tag is pushed; release workflow only runs on tag pushes.
- Every code task ends with `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- TDD exception: dependency, version, documentation and CI changes have no production behavior; their gate is the existing 118-test suite plus the stated metadata command.

## Review Focus

- Bare `vsrmi` starts in Normal mode; welcome stays until actual text exists, and `i` is never swallowed.
- An existing but zero-byte file (`vsrmi empty.rs`) must not show welcome because its path is set.
- Narrow terminals must not let the status bar's right side overwrite the file name; welcome must not panic at tiny sizes.
- New language rules must not miscolour comments or strings: `#` inside a TOML string is not a comment, `$` inside a double-quoted shell string is not a separate constant.
- Version strings must agree everywhere a user sees them: `Cargo.toml`, `vsrmi --version`, `CHANGELOG.md`, README badge and release-workflow tag check.

---

### Task 1: Update dependencies

**Files:**
- Modify: `Cargo.toml:34` (`dirs`), `Cargo.toml:39` (`arboard`)
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: nothing.
- Produces: locked dependency graph used by every later task.

- [ ] **Step 1: Bump the two direct dependencies**

In `Cargo.toml`, change `dirs = "6"` to `dirs = "7"` and `arboard = { version = "3.6", default-features = false }` to `arboard = { version = "3.7", default-features = false }`.

- [ ] **Step 2: Update the lockfile**

Run: `cargo update`

Expected: `Locking ... to highest Rust 1.88 compatible versions`, and `clipboard-win` moves off yanked `5.4.1` to `5.4.3` (or newer non-yanked).

- [ ] **Step 3: Verify the new graph compiles and the suite is green**

Run:
```sh
cargo check --all-targets
cargo test
```

Expected: `Finished` with no errors; `118 passed; 0 failed`.

- [ ] **Step 4: Fallback only if `dirs` 7 rejects Rust 1.88**

If the compiler reports an MSRV/newer-language error for `dirs 7`, set `dirs = "6"`, run `cargo update -p dirs --precise 6.0.0`, rerun Step 3, and note in the commit body and final changelog task that `dirs` stayed at 6 because 7 is not Rust 1.88 compatible.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): update locked dependencies and platform crates"
```

---

### Task 2: Align version, changelog and repository attributes

**Files:**
- Modify: `Cargo.toml:3`, `Cargo.toml:5`
- Modify: `.gitattributes`
- Replace: `CHANGELOG.md`

**Interfaces:**
- Consumes: nothing.
- Produces: `CARGO_PKG_VERSION == "0.1.0"` for the welcome screen and help title; changelog links point at `Lefkee/Vsrmi`.

- [ ] **Step 1: Bump the manifest and fix the stale comment**

In `Cargo.toml`:
- `version = "0.0.1"` → `version = "0.1.0"`.
- The comment line `# Edition 2026 alone needs 1.85, ...` → `# Edition 2024 alone needs 1.85, ...`.

- [ ] **Step 2: Stop hiding lockfile diffs**

In `.gitattributes`, change:
```text
Cargo.lock linguist-generated=true -diff
```
to:
```text
Cargo.lock linguist-generated=true
```

- [ ] **Step 3: Rewrite `CHANGELOG.md` as one 0.1.0 release**

Use this exact structure and keep the existing English prose style:

```markdown
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
```

If Task 1 fell back to `dirs` 6, add one line under `### Changed`:
`- \`dirs\` stayed at 6 because version 7 requires a newer Rust than the 1.88 MSRV.`

- [ ] **Step 4: Verify metadata and attributes**

Run:
```sh
cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])'
git check-attr diff Cargo.lock
cargo check --all-targets
```

Expected: `0.1.0`; `git check-attr` does **not** print `diff: unset`; `Finished`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml .gitattributes CHANGELOG.md
git commit -m "chore(repo): prepare 0.1.0 metadata and changelog"
```

---

### Task 3: Add the blank-scratch document predicate

**Files:**
- Modify: `src/editor/document/mod.rs` (impl block and tests)
- Modify: `src/app/state.rs:235-239` (delegate or remove duplicate logic)

**Interfaces:**
- Consumes: nothing.
- Produces: `Document::is_blank_scratch(&self) -> bool`; true only when `path.is_none() && !dirty && text.len_chars() == 0`.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` in `src/editor/document/mod.rs`:

```rust
#[test]
fn blank_scratch_is_unnamed_empty_and_clean() {
    let mut document = Document::new();
    assert!(document.is_blank_scratch());

    document.insert(0, "x");
    assert!(!document.is_blank_scratch());
}

#[test]
fn a_named_empty_document_is_not_a_blank_scratch() {
    let document = Document::from_text("", Some(std::path::PathBuf::from("empty.rs")));
    assert!(!document.is_blank_scratch());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test blank_scratch a_named_empty -- --nocapture`

Expected: compile error, `no method named is_blank_scratch`.

- [ ] **Step 3: Implement the predicate**

Add this method next to `display_name` in `src/editor/document/mod.rs`:

```rust
/// Whether this is the untouched scratch document shown on a bare start.
#[must_use]
pub fn is_blank_scratch(&self) -> bool {
    self.path.is_none() && !self.dirty && self.text.len_chars() == 0
}
```

Then make `App::is_scratch` delegate so the condition exists once:

```rust
fn is_scratch(&self, index: usize) -> bool {
    self.buffers[index].document.is_blank_scratch()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test blank_scratch a_named_empty`

Expected: both tests pass; the rest of the suite stays green.

- [ ] **Step 5: Commit**

```bash
git add src/editor/document/mod.rs src/app/state.rs
git commit -m "feat(document): add blank scratch predicate"
```

---

### Task 4: Add the welcome widget

**Files:**
- Create: `src/ui/widgets/welcome.rs`
- Modify: `src/ui/widgets/mod.rs`
- Test: inline `#[cfg(test)] mod tests` in `welcome.rs`

**Interfaces:**
- Consumes: `Theme` slots `text`, `gutter`, `syntax.keyword`, `syntax.function`, `popup_border`.
- Produces: `pub struct Welcome<'a> { pub version: &'a str, pub theme: &'a Theme }`, implementing `ratatui::widgets::Widget`.

- [ ] **Step 1: Register the module and write the failing tests**

In `src/ui/widgets/mod.rs`, add:
```rust
pub mod welcome;
pub use welcome::Welcome;
```

Create `src/ui/widgets/welcome.rs` with the struct and a `render` that returns immediately, plus these tests:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test welcome`

Expected: tests compile but fail because the stub renders nothing (`missing vsrmi`).

- [ ] **Step 3: Implement the widget**

Use these constants and layout:

```rust
const FULL_MIN_WIDTH: u16 = 60;
const FULL_MIN_HEIGHT: u16 = 15;
const PANEL_WIDTH: u16 = 56;
const PANEL_HEIGHT: u16 = 13;
const COMPACT_MIN_WIDTH: u16 = 20;
```

Implementation rules:
- `render` starts with `Clear.render(area, surface)` then `surface.set_style(area, self.theme.text)`, so the blank document's line-number gutter cannot show through.
- Full layout only when `area.width >= FULL_MIN_WIDTH && area.height >= FULL_MIN_HEIGHT`; centre a 56×13 rounded `Block` with `border_style(self.theme.popup_border)` and `style(self.theme.text)`.
- Full content is exactly 11 inner lines, centred with `Paragraph::new(lines).alignment(Alignment::Center)`:
  1. blank
  2. `vsrmi` + two spaces + `v{version}`
  3. `a modal terminal code editor`
  4. blank
  5. `i` / `insert mode`
  6. `:help` / `every key`
  7. `Ctrl+B` / `file tree`
  8. `Ctrl+S` / `save`
  9. blank
  10. `Press i to start · :e <file> to open`
  11. blank
- Shortcut lines use an 8-column key field (`format!("{key:<8}")`) in `theme.syntax.function`; labels use `theme.text`.
- The wordmark uses `theme.syntax.keyword` plus `Modifier::BOLD`; version, tagline and hint use `theme.gutter`.
- Compact layout when `area.width >= COMPACT_MIN_WIDTH && area.height >= 2`: two centred lines, `vsrmi v{version}` and `i to start · :help · Ctrl+B`.
- Smaller than compact: draw nothing after the clear/fill.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test welcome`

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ui/widgets/welcome.rs src/ui/widgets/mod.rs
git commit -m "feat(ui): add welcome widget"
```

---

### Task 5: Render the welcome screen on a bare start

**Files:**
- Modify: `src/ui/mod.rs` (imports, `draw`)

**Interfaces:**
- Consumes: `Document::is_blank_scratch`, `Welcome`.
- Produces: `ui::draw` renders welcome on `regions.editor` and hides the editor caret while it is visible.

- [ ] **Step 1: Wire the predicate and widget**

In `src/ui/mod.rs`:
- Add `Welcome` to the `use crate::ui::widgets::{...}` list.
- At the start of `draw`, compute `let welcome = app.buffer().document.is_blank_scratch();`.
- Keep `editor_caret` only when not showing welcome:
  `let editor_caret = (!welcome).then(|| editor.caret_position(regions.editor));`
- After `frame.render_widget(editor, regions.editor);`, render:
```rust
if welcome {
    frame.render_widget(
        Welcome {
            version: env!("CARGO_PKG_VERSION"),
            theme: &app.theme,
        },
        regions.editor,
    );
}
```
- Leave the later `prompt_caret.or(editor_caret)` logic unchanged; `editor_caret` is now an `Option`.

- [ ] **Step 2: Verify the build and tests**

Run:
```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: clippy clean; all tests pass.

- [ ] **Step 3: Smoke-test the real binary**

Run:
```sh
cargo build --release --locked
timeout 2 script -qec "target/release/vsrmi" /dev/null > /tmp/opencode/vsrmi-smoke.txt || true
grep -a -m1 "vsrmi" /tmp/opencode/vsrmi-smoke.txt
grep -a -m1 "Press i to start" /tmp/opencode/vsrmi-smoke.txt
```

Expected: both greps find text. If the terminal capture does not expose the welcome text, record that limitation in the task notes instead of changing the widget.

- [ ] **Step 4: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat(ui): show welcome on empty startup"
```

---

### Task 6: Add TOML syntax highlighting

**Files:**
- Create: `src/syntax/languages/toml.rs`
- Modify: `src/syntax/languages/mod.rs`
- Test: `src/syntax/engine.rs` tests

**Interfaces:**
- Consumes: `Language`, `HighlightKind`.
- Produces: `pub static TOML: Language`; `syntax::detect` resolves `.toml` and `Cargo.lock`.

- [ ] **Step 1: Add the shared span helper and failing TOML test**

In `src/syntax/engine.rs` tests, add:

```rust
fn kind_at(language: &str, line: &str, char_index: usize) -> Option<HighlightKind> {
    let highlighter = by_name(language).expect("known language");
    let (spans, _) = highlighter.highlight_line(line, BlockState::Normal);
    spans
        .iter()
        .find(|span| span.start <= char_index && char_index < span.end)
        .map(|span| span.kind)
}
```

Add `use std::path::Path;` and this test:

```rust
#[test]
fn toml_tables_keys_strings_and_comments() {
    assert_eq!(kind_at("toml", "[package]", 1), Some(HighlightKind::Heading));

    let line = "name = \"vsrmi\" # note";
    assert_eq!(kind_at("toml", line, 1), Some(HighlightKind::Attribute));
    assert_eq!(kind_at("toml", line, 8), Some(HighlightKind::String));
    assert_eq!(kind_at("toml", line, 15), Some(HighlightKind::Comment));

    assert_eq!(
        kind_at("toml", "url = 'https://example.com'", 7),
        Some(HighlightKind::String)
    );
    assert_eq!(
        crate::syntax::detect(Path::new("Cargo.lock")).map(|h| h.language.name),
        Some("toml")
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test toml_tables_keys_strings_and_comments`

Expected: fail because no `toml` highlighter exists.

- [ ] **Step 3: Add the TOML definition and register it**

Create `src/syntax/languages/toml.rs` with:

```rust
name: "toml",
extensions: &["toml"],
filenames: &["cargo.lock"],
keywords: &[],
types: &[],
constants: &["true", "false", "inf", "nan"],
line_comment: Some("#"),
block_comment: None,
nested_block_comments: false,
macro_suffix: false,
capitalised_types: false,
extra_rules: &[
    (r"^[ \t]*\[\[?[^\]]+\]\]?", HighlightKind::Heading),
    (r"^[ \t]*[A-Za-z0-9_.-]+", HighlightKind::Attribute),
    (r"'[^']*'", HighlightKind::String),
],
prose: false,
snippets: &[],
```

In `src/syntax/languages/mod.rs`, add `pub mod toml;` and `&toml::TOML` at the end of `all()`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test toml_tables_keys_strings_and_comments`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/syntax/languages/toml.rs src/syntax/languages/mod.rs src/syntax/engine.rs
git commit -m "feat(syntax): add toml highlighting"
```

---

### Task 7: Add JSON syntax highlighting

**Files:**
- Create: `src/syntax/languages/json.rs`
- Modify: `src/syntax/languages/mod.rs`
- Test: `src/syntax/engine.rs` tests

**Interfaces:**
- Consumes: `kind_at` helper from Task 6.
- Produces: `pub static JSON: Language`; `.json` detection.

- [ ] **Step 1: Write the failing JSON test**

Add to `src/syntax/engine.rs` tests:

```rust
#[test]
fn json_keys_and_literals() {
    let line = "{\"ok\": true}";
    assert_eq!(kind_at("json", line, 2), Some(HighlightKind::Attribute));
    assert_eq!(kind_at("json", line, 7), Some(HighlightKind::Constant));

    assert_eq!(
        kind_at("json", "{\"k\": \"v\"}", 7),
        Some(HighlightKind::String)
    );
    assert_eq!(
        crate::syntax::detect(Path::new("data.json")).map(|h| h.language.name),
        Some("json")
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test json_keys_and_literals`

Expected: fail because no `json` highlighter exists.

- [ ] **Step 3: Add the JSON definition and register it**

Create `src/syntax/languages/json.rs` with:

```rust
name: "json",
extensions: &["json"],
filenames: &[],
keywords: &[],
types: &[],
constants: &["true", "false", "null"],
line_comment: None,
block_comment: None,
nested_block_comments: false,
macro_suffix: false,
capitalised_types: false,
extra_rules: &[(r#"^[ \t]*"[^"]*"[ \t]*:"#, HighlightKind::Attribute)],
prose: false,
snippets: &[],
```

In `src/syntax/languages/mod.rs`, add `pub mod json;` and `&json::JSON` at the end of `all()`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test json_keys_and_literals`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/syntax/languages/json.rs src/syntax/languages/mod.rs src/syntax/engine.rs
git commit -m "feat(syntax): add json highlighting"
```

---

### Task 8: Add YAML syntax highlighting

**Files:**
- Create: `src/syntax/languages/yaml.rs`
- Modify: `src/syntax/languages/mod.rs`
- Test: `src/syntax/engine.rs` tests

**Interfaces:**
- Consumes: `kind_at` helper from Task 6.
- Produces: `pub static YAML: Language`; `.yaml`/`.yml` detection.

- [ ] **Step 1: Write the failing YAML test**

Add to `src/syntax/engine.rs` tests:

```rust
#[test]
fn yaml_keys_markers_and_comments() {
    let line = "name: value # note";
    assert_eq!(kind_at("yaml", line, 1), Some(HighlightKind::Attribute));
    assert_eq!(kind_at("yaml", line, 12), Some(HighlightKind::Comment));

    assert_eq!(kind_at("yaml", "---", 0), Some(HighlightKind::Heading));
    assert_eq!(kind_at("yaml", "&anchor", 1), Some(HighlightKind::Constant));
    assert_eq!(kind_at("yaml", "!!str", 1), Some(HighlightKind::Type));
    assert_eq!(
        crate::syntax::detect(Path::new("config.yml")).map(|h| h.language.name),
        Some("yaml")
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test yaml_keys_markers_and_comments`

Expected: fail because no `yaml` highlighter exists.

- [ ] **Step 3: Add the YAML definition and register it**

Create `src/syntax/languages/yaml.rs` with:

```rust
name: "yaml",
extensions: &["yaml", "yml"],
filenames: &[],
keywords: &[],
types: &[],
constants: &["true", "false", "null"],
line_comment: Some("#"),
block_comment: None,
nested_block_comments: false,
macro_suffix: false,
capitalised_types: false,
extra_rules: &[
    (r"^(?:---|\.\.\.)[ \t]*$", HighlightKind::Heading),
    (r"^[ \t]*(?:-[ \t]+)?[A-Za-z0-9_.-]+", HighlightKind::Attribute),
    (r"'[^']*'", HighlightKind::String),
    (r"[&*][A-Za-z0-9_-]+", HighlightKind::Constant),
    (r"!![A-Za-z0-9_-]+", HighlightKind::Type),
],
prose: false,
snippets: &[],
```

In `src/syntax/languages/mod.rs`, add `pub mod yaml;` and `&yaml::YAML` at the end of `all()`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test yaml_keys_markers_and_comments`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/syntax/languages/yaml.rs src/syntax/languages/mod.rs src/syntax/engine.rs
git commit -m "feat(syntax): add yaml highlighting"
```

---

### Task 9: Add Shell syntax highlighting

**Files:**
- Create: `src/syntax/languages/shell.rs`
- Modify: `src/syntax/languages/mod.rs`
- Test: `src/syntax/engine.rs` tests

**Interfaces:**
- Consumes: `kind_at` helper from Task 6.
- Produces: `pub static SHELL: Language`; `.sh`/`.bash`/`.zsh`/`.fish`/`.ksh` and dotfile detection.

- [ ] **Step 1: Write the failing Shell test**

Add to `src/syntax/engine.rs` tests:

```rust
#[test]
fn shell_keywords_variables_and_comments() {
    let line = "if [ -n $HOME ]; then";
    assert_eq!(kind_at("shell", line, 0), Some(HighlightKind::Keyword));
    assert_eq!(kind_at("shell", line, 8), Some(HighlightKind::Constant));
    assert_eq!(kind_at("shell", line, 17), Some(HighlightKind::Keyword));

    // A variable inside double quotes is part of the string, not a separate
    // constant; the engine has no lookahead to split it.
    assert_eq!(
        kind_at("shell", "echo \"$HOME\"", 6),
        Some(HighlightKind::String)
    );
    assert_eq!(kind_at("shell", "# note", 0), Some(HighlightKind::Comment));
    assert_eq!(kind_at("shell", "VALUE=1", 0), Some(HighlightKind::Attribute));
    assert_eq!(
        crate::syntax::detect(Path::new("build.sh")).map(|h| h.language.name),
        Some("shell")
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test shell_keywords_variables_and_comments`

Expected: fail because no `shell` highlighter exists.

- [ ] **Step 3: Add the Shell definition and register it**

Create `src/syntax/languages/shell.rs` with:

```rust
name: "shell",
extensions: &["sh", "bash", "zsh", "fish", "ksh"],
filenames: &[".bashrc", ".bash_profile", ".zshrc", ".profile", ".bash_aliases"],
keywords: &[
    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done",
    "case", "esac", "in", "function", "select", "time", "return", "break",
    "continue", "local", "export", "readonly", "declare", "unset",
],
types: &[],
constants: &["true", "false"],
line_comment: Some("#"),
block_comment: None,
nested_block_comments: false,
macro_suffix: false,
capitalised_types: false,
extra_rules: &[
    (r"\$\{?[A-Za-z_][A-Za-z0-9_]*\}?", HighlightKind::Constant),
    (r"\$[0-9@*#?$!-]", HighlightKind::Constant),
    (r"^[ \t]*[A-Za-z_][A-Za-z0-9_]*=", HighlightKind::Attribute),
    (r"^[ \t]*(?:function[ \t]+)?[A-Za-z_][A-Za-z0-9_-]*[ \t]*\(\)", HighlightKind::Function),
],
prose: false,
snippets: &[],
```

In `src/syntax/languages/mod.rs`, add `pub mod shell;` and `&shell::SHELL` at the end of `all()`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test shell_keywords_variables_and_comments`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/syntax/languages/shell.rs src/syntax/languages/mod.rs src/syntax/engine.rs
git commit -m "feat(syntax): add shell highlighting"
```

---

### Task 10: Make the status bar width-aware and tidy its docs

**Files:**
- Modify: `src/ui/widgets/status_bar.rs`
- Test: inline `#[cfg(test)] mod tests` in `status_bar.rs`

**Interfaces:**
- Consumes: existing `StatusBar` fields and `Theme`.
- Produces: same public `StatusBar` API; rendering omits the right block when it would collide.

- [ ] **Step 1: Write the failing tests**

Add at the end of `src/ui/widgets/status_bar.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test status_bar`

Expected: `single_cursor_does_not_leave_an_empty_segment` fails (`LF    Ln` exists), and the narrow test fails because `rust` is drawn over the name area.

- [ ] **Step 3: Rebuild the right-hand line and make it width-aware**

In `StatusBar::render`:
- Build a `Vec<String>` for the right side: language, line-ending label, `"{n} cursors"` only when `cursor_count > 1`, `"Ln {line} Col {col}"`, and `"{icon}{pct}%"`.
- Join with two spaces and wrap once: `Line::from(Span::styled(format!(" {} ", segments.join("  ")), base)).right_aligned()`.
- Render as:
```rust
let left = Line::from(left);
let show_right = left.width() + right.width() <= usize::from(area.width);
left.render(area, surface);
if show_right {
    right.render(area, surface);
}
```
- Keep mode colours and labels unchanged.

- [ ] **Step 4: Translate the module docs and field comments to English**

Replace the Turkish module header and every Turkish field/method comment in this file with concise English equivalents; match the style of `src/ui/widgets/tabs.rs`. Keep `mode_label` text and icons unchanged.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test status_bar`

Expected: both tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/ui/widgets/status_bar.rs
git commit -m "fix(ui): keep status bar readable on narrow terminals"
```

---

### Task 11: Refresh the help popup

**Files:**
- Modify: `src/app/commands.rs` (`HELP`, `Command::Help` arm)
- Test: inline `#[cfg(test)] mod tests` in `commands.rs`

**Interfaces:**
- Consumes: `CARGO_PKG_VERSION`.
- Produces: `HELP` includes the modern key set and a `LANGUAGES` line; popup title is `Vsrmi <version>`.

- [ ] **Step 1: Write the failing test**

Add at the end of `src/app/commands.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::HELP;

    #[test]
    fn help_lists_modern_keys_and_languages() {
        for needle in [
            "Ctrl+B", "Ctrl+Q", "Ctrl+R", "Alt+", "LANGUAGES", "toml", "yaml", "shell",
            ":help",
        ] {
            assert!(HELP.contains(needle), "missing {needle}");
        }
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test help_lists_modern_keys_and_languages`

Expected: fail with `missing LANGUAGES` (or the first absent key).

- [ ] **Step 3: Replace `HELP` and the title**

Use a body that fits 80 columns and keeps the existing section layout. It must contain every tested needle and these lines:

```text
MODES     i insert    v visual    V line    : command    / search

MOVE      h j k l     w b e words     0 ^ $ line     gg G file
EDIT      x delete    dd line     yy yank    p paste    u undo
          Ctrl+R redo     Alt+↑/↓ add cursor     Esc collapse
SEARCH    / next      ? previous      n / N repeat
FILES     Ctrl+B tree     Ctrl+N / Ctrl+P buffers     Ctrl+S save
          Ctrl+Q quit     Tab accept completion

COMMANDS  :w [path]  :q[!]  :wq  :e[!] path  :bn  :bp  :help
          :set <option> [value]     :theme <name>     :<line>
          :%s/pattern/replacement/g

LANGUAGES rust c cpp zig python markdown toml json yaml shell
```

Change the command arm to:
```rust
Command::Help => app.show_popup(
    format!("Vsrmi {}", env!("CARGO_PKG_VERSION")),
    HELP,
),
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test help_lists_modern_keys_and_languages`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/commands.rs
git commit -m "docs(help): list modern keys and supported languages"
```

---

### Task 12: Refresh the README and preview

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: new language names, help keys, version 0.1.0.
- Produces: user-facing documentation consistent with the binary.

- [ ] **Step 1: Add badges and a real-output preview**

At the top, replace the two existing badges with four:

```markdown
[![CI](https://github.com/Lefkee/Vsrmi/actions/workflows/ci.yml/badge.svg)](https://github.com/Lefkee/Vsrmi/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![Version: 0.1.0](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/Lefkee/Vsrmi/releases)
```

After the intro block, add a `## 👀 Önizleme` section containing a fenced `text` block that shows the welcome panel and the status bar exactly as designed:

```text
┌──────────────────────────────────────────────────┐
│                                                  │
│                  vsrmi  v0.1.0                   │
│          a modal terminal code editor            │
│                                                  │
│          i        insert mode                    │
│          :help    every key                      │
│          Ctrl+B   file tree                      │
│          Ctrl+S   save                           │
│                                                  │
│      Press i to start · :e <file> to open        │
│                                                  │
└──────────────────────────────────────────────────┘
```

Add the status bar sample directly below, using the existing sample but with the corrected right side:
```text
◆ NORMAL ▌ dosyaadi.py ●                 python  LF  Ln 12 Col 8  ▅42%
```

- [ ] **Step 2: Update the language table**

Add these rows to the existing table:

| Uzantı | Dil |
|---|---|
| `.toml` `Cargo.lock` | TOML |
| `.json` | JSON |
| `.yaml` `.yml` | YAML |
| `.sh` `.bash` `.zsh` `.fish` `.ksh` | Shell |

Update the file-tree label table so `[cf]` and `[sh]` are described by the real languages they now map to; if the tree widget uses different labels, read `src/ui/widgets/file_tree.rs` and make the README match the code exactly.

- [ ] **Step 3: Update the key reference**

Add the missing bindings from `:help`:
- `gg` `G`, `0` `^` `$` under movement.
- `Ctrl+R` redo, `Alt+↑` `Alt+↓` multi-cursor, `Tab` completion under editing.
- `Ctrl+Q` quit under files.
- `:help` under commands.

Keep the existing Turkish labels and table shape.

- [ ] **Step 4: Fix links and remove dead reference definitions**

- Keep `[Termi]: https://github.com/tuna4ll/termi` as the legitimate fork attribution.
- Remove `[releases]` and `[CONTRIBUTING.md]` reference definitions if they are not linked in the body; otherwise link the matching text.
- Verify with `grep -n '\[' README.md` that no dead definitions remain.

- [ ] **Step 5: Verify packaging still accepts the README**

Run: `cargo package --locked --allow-dirty --no-verify`

Expected: `Packaged ... files`, no missing-README error.

- [ ] **Step 6: Commit**

```bash
git add README.md
git commit -m "docs(readme): refresh preview, badges and language support"
```

---

### Task 13: Add Dependabot and a packaging CI job

**Files:**
- Create: `.github/dependabot.yml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: existing CI structure.
- Produces: weekly Cargo/Actions updates; `cargo package --locked` in CI.

- [ ] **Step 1: Add Dependabot**

Create `.github/dependabot.yml`:

```yaml
version: 2

updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5

  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
```

- [ ] **Step 2: Add the package job**

Append to `.github/workflows/ci.yml`:

```yaml
  package:
    name: package
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      # Checks the published crate: README, license, excludes and the files
      # cargo would actually upload. A clean checkout has no dirty state.
      - run: cargo package --locked
```

- [ ] **Step 3: Validate YAML syntax**

Run:
```sh
python3 -c 'import yaml; yaml.safe_load(open(".github/dependabot.yml")); yaml.safe_load(open(".github/workflows/ci.yml")); print("yaml ok")'
```

Expected: `yaml ok`.

- [ ] **Step 4: Commit**

```bash
git add .github/dependabot.yml .github/workflows/ci.yml
git commit -m "ci: add dependabot and package check"
```

---

### Task 14: Final verification and push

**Files:**
- No source changes expected.

**Interfaces:**
- Consumes: all previous tasks.
- Produces: a green `main` pushed to `origin/main`; no tag pushed.

- [ ] **Step 1: Run the full gate**

Run:
```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --locked
cargo package --locked --allow-dirty
```

Expected: format clean; clippy `Finished`; all tests pass (118 existing + new welcome/document/syntax/status/help tests); release build `Finished`; package succeeds.

- [ ] **Step 2: Verify user-visible version and help**

Run:
```sh
target/release/vsrmi --version
target/release/vsrmi --help | head -5
```

Expected: `Vsrmi 0.1.0`; usage mentions `vsrmi [OPTIONS] [FILE]...`.

- [ ] **Step 3: Verify the version is consistent**

Run:
```sh
grep -n '^version' Cargo.toml
grep -n '0.1.0' CHANGELOG.md README.md .github/workflows/release.yml
cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])'
```

Expected: manifest and metadata `0.1.0`; changelog and README mention 0.1.0. The release workflow has no hardcoded version (it compares the tag to the manifest).

- [ ] **Step 4: Verify no tag or force-push is needed**

Run:
```sh
git status -sb
git log --oneline -5
git tag -l
```

Expected: clean tree; `main...origin/main` with only local commits ahead; no new tags.

- [ ] **Step 5: Push**

```bash
git push origin main
```

Expected: normal fast-forward push; no `--force`.

- [ ] **Step 6: Confirm remote sync**

Run:
```sh
git fetch origin
git status -sb
git rev-parse HEAD origin/main
```

Expected: both hashes equal; `## main...origin/main`.

---

## Self-Review Notes

- Spec coverage: welcome (Tasks 3–5), four languages (6–9), status bar (10), help (11), dependencies/version/changelog (1–2), README (12), CI/Dependabot/package (13), verification/push (14).
- Review-focus tests exist in: Task 3 (named empty file), Task 4 (tiny/narrow welcome), Task 10 (narrow status bar), Tasks 6–9 (string/comment/variable boundaries), Tasks 2 and 14 (version consistency).
- Type names are consistent across tasks: `Document::is_blank_scratch`, `Welcome { version, theme }`, `kind_at`, unchanged `StatusBar` fields, private `HELP` const.
- No task adds a command palette, persistence, new theme slot or engine change.
