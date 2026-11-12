# Contributing

Thanks for taking a look. This is a small project; the bar is that a change
leaves the codebase easier to read than it found it.

## Before you open a pull request

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs exactly these, plus the test suite on Linux, macOS and Windows and a
build against the minimum supported Rust version. All of it has to pass.

## What the code expects of you

- **One responsibility per module.** Every module starts with a header stating
  its purpose, its responsibility and its public API. If your change makes that
  header inaccurate, the change is in the wrong module.
- **The editor core stays UI-free.** Nothing under `src/editor/` may know about
  terminals, keys or colours. Rendering reads state and never mutates it, with
  the two documented exceptions in `ui::draw`.
- **Comment the why, not the what.** The existing comments explain trade-offs
  and non-obvious constraints. Comments that restate the code will be asked
  about in review.
- **No `unsafe`.** It is forbidden crate-wide in `Cargo.toml`.
- **Character indices, not bytes.** Byte offsets break on UTF-8 and screen
  columns break on tabs; conversions belong at the edges, in `ui::text` and the
  rope accessors.

## Commits

Conventional-commit style, scoped to the module you touched:

```
feat(search): add whole-word matching
fix(ui): keep the caret visible when the gutter width changes
```

Small commits that each leave the tree building are much easier to review than
one large one.

## Minimum supported Rust version

Currently 1.88, enforced in CI. Raising it is fine when there is a reason —
update `rust-version` in `Cargo.toml` and the `msrv` job in
`.github/workflows/ci.yml` together, and say so in the changelog.

## Adding a language to the highlighter

Add a file under `src/syntax/languages/`, export a `&'static Language`, and add
one line to `all()`. No engine or rendering change should be necessary; if one
is, that is worth discussing in the issue first.
