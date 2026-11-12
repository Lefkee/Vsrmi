//! TOML language definition.
//!
//! TOML is mostly structure rather than keywords: table headers, dotted keys
//! and two string syntaxes. The extra rules anchor the table and key patterns
//! to the start of the line so a value that happens to look like a key is not
//! coloured as one.

use crate::syntax::{HighlightKind, Language};

/// TOML.
pub static TOML: Language = Language {
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
};
