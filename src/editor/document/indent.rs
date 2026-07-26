//! # Indentation
//!
//! **Purpose:** answer "how is this line indented, and how should the next one
//! be?".
//!
//! **Responsibility:** pure functions over a [`Document`]. No language grammar
//! is involved — the heuristic is the classic one (copy the previous line's
//! indent, add a level after an opening bracket, remove one before a closing
//! bracket), which is what a regex-era editor can honestly do and what most
//! users expect while typing.
//!
//! **Public API:** [`first_non_blank`], [`indent_of`], [`indent_unit`],
//! [`auto_indent_for_new_line`], [`should_dedent`].

use ropey::RopeSlice;

use crate::editor::document::Document;

/// Column of the first non-whitespace character on `line`.
///
/// Returns the line length for blank lines, which is where the caret belongs
/// when jumping "home" on an empty line.
#[must_use]
pub fn first_non_blank(doc: &Document, line: usize) -> usize {
    let slice = doc.line(line);
    slice
        .chars()
        .position(|ch| !ch.is_whitespace())
        .unwrap_or_else(|| doc.line_len(line))
}

/// The leading whitespace of `line`, copied verbatim.
///
/// Copying rather than recomputing preserves whatever the file already uses, so
/// editing a tab-indented file with `use_spaces = true` does not produce a mix.
#[must_use]
pub fn indent_of(doc: &Document, line: usize) -> String {
    doc.line(line)
        .chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .collect()
}

/// One level of indentation.
#[must_use]
pub fn indent_unit(tab_width: usize, use_spaces: bool) -> String {
    if use_spaces {
        " ".repeat(tab_width)
    } else {
        "\t".to_string()
    }
}

/// The indentation a line inserted after `line` should start with.
///
/// `col` is where the split happens, so that pressing Enter in the middle of a
/// line only considers the part that stays behind.
#[must_use]
pub fn auto_indent_for_new_line(
    doc: &Document,
    line: usize,
    col: usize,
    tab_width: usize,
    use_spaces: bool,
) -> String {
    // An indent longer than the split point means we split inside the leading
    // whitespace; the new line cannot inherit more than it had.
    let mut indent: String = indent_of(doc, line).chars().take(col).collect();

    if opens_block(doc.line(line), col) {
        indent.push_str(&indent_unit(tab_width, use_spaces));
    }
    indent
}

/// Whether the text of `slice` up to `col` ends with an unclosed opening
/// bracket, ignoring trailing whitespace.
fn opens_block(slice: RopeSlice<'_>, col: usize) -> bool {
    slice
        .chars()
        .take(col)
        .filter(|ch| !ch.is_whitespace())
        .last()
        .is_some_and(|ch| matches!(ch, '{' | '[' | '(' | ':'))
}

/// Whether typing `ch` should remove one level of indentation first.
///
/// True only for a closing bracket typed as the first non-blank character of an
/// indented line — that is the moment a human would reach for backspace, and
/// doing it automatically is what makes `}` land under its opener.
#[must_use]
pub fn should_dedent(doc: &Document, line: usize, col: usize, ch: char) -> bool {
    matches!(ch, '}' | ']' | ')')
        && col > 0
        // Nothing but whitespace to the left.
        && first_non_blank(doc, line) >= col
}
