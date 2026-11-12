//! YAML language definition.
//!
//! YAML is structural: a key is whatever sits before the colon, and the tokens
//! that carry meaning are the document markers, anchors/aliases and tags. The
//! key rule is anchored to the start of the line, where YAML keys live, while
//! the marker rules can appear anywhere on the line.

use crate::syntax::{HighlightKind, Language};

/// YAML.
pub static YAML: Language = Language {
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
        (
            r"^[ \t]*(?:-[ \t]+)?[A-Za-z0-9_.-]+",
            HighlightKind::Attribute,
        ),
        (r"'[^']*'", HighlightKind::String),
        (r"[&*][A-Za-z0-9_-]+", HighlightKind::Constant),
        (r"!![A-Za-z0-9_-]+", HighlightKind::Type),
    ],
    prose: false,
    snippets: &[],
};
