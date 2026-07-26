# termi

A modal terminal code editor written in Rust, built on [ratatui] and
[crossterm]. Inspired by Helix and Kilo: modal like the first, small enough to
read end to end like the second.

```
cargo run --release -- src/main.rs
```

## What it does

**Text** — UTF-8 throughout, a [ropey] rope underneath, so editing a large file
costs the same as editing a small one. Tabs, wide glyphs and mixed line endings
are handled at the edges: everything in between works in plain character
indices.

**Modal editing** — normal, insert, visual, visual-line, command and search
modes. `hjkl` and word motions, `dd`/`yy`/`p`, undo and redo with typing merged
into sensible steps, and multiple cursors (`Alt+↑` / `Alt+↓`) as a first-class
part of the editing core rather than a bolted-on mode.

**Search** — incremental, literal or regex, smart case, with matches highlighted
as you type and `:%s/a/b/g` for replacement.

**Syntax highlighting** — regex based, for Rust, C, C++, Zig, Python and
Markdown. Block-comment state is cached per line, so scrolling deep into a file
does not rescan it.

**Files** — multiple buffers with a tab strip, a lazily expanded file tree
(`Ctrl+B`), atomic saves, and a watcher that reloads clean buffers when they
change on disk and warns rather than clobbers when they do not.

**Looks** — dark and light themes built in, plus TOML themes that override only
the slots you care about.

## Keys

Press `:help` inside the editor for the same list.

| | |
|---|---|
| `i` `a` `I` `A` `o` `O` | enter insert mode |
| `v` `V` | character-wise / line-wise visual mode |
| `h j k l` `w b e` `0 ^ $` `gg G` | motions |
| `x` `dd` `yy` `p` `u` `Ctrl+R` | delete, yank, paste, undo, redo |
| `/` `?` `n` `N` | search forwards, backwards, repeat |
| `Alt+↑` `Alt+↓` `Esc` | add a cursor above/below, collapse to one |
| `Ctrl+B` | file tree |
| `Ctrl+N` `Ctrl+P` | next / previous buffer |
| `Ctrl+S` `Ctrl+Q` | save, quit |
| `:` | command line |

Commands: `:w [path]` `:q[!]` `:wq` `:e[!] path` `:bn` `:bp` `:<line>`
`:set <option> [value]` `:theme <name>` `:%s/pattern/replacement/g`

## Configuration

See [`config.example.toml`](config.example.toml). Themes go in
`<config-dir>/termi/themes/<name>.toml` and layer over a built-in base:

```toml
name = "midnight"
base = "dark"

[comment]
fg = "#4a5058"
italic = true

[selection]
bg = "bright-blue"
```

## Architecture

Layers depend downwards only:

```
app/         event loop, state, action dispatch, ex commands
├── ui/      layout and widgets; renderer/ owns the terminal
├── input/   keys → actions
└── editor/  text, with no knowledge of terminals
    ├── document/   rope, file, dirty state, indentation
    ├── cursor/     positions, motions, word boundaries
    ├── selection/  character ranges
    ├── buffer/     document + cursors + viewport + history
    └── command/    ex-command parsing

config/  theme/  syntax/  search/  undo/  clipboard/  filesystem/
```

The editor core is UI-free and the UI layer is read-only, so a render pass is a
pure function of the state plus the terminal size. Every module's header
documents its purpose, its responsibility and its public API.

`unsafe_code` is forbidden crate-wide.

## Development

```
cargo test
cargo clippy --all-targets
cargo fmt
```

## Roadmap

Syntax highlighting is deliberately regex based for now. The next step is
tree-sitter behind the same `Highlight` span interface, which the renderer and
themes already consume — no changes above the `syntax` module. Mouse support is
also intentionally deferred.

## License

MIT

[ratatui]: https://ratatui.rs
[crossterm]: https://github.com/crossterm-rs/crossterm
[ropey]: https://github.com/cessen/ropey
