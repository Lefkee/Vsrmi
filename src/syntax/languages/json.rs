//! JSON language definition.
//!
//! JSON has no keywords and no comments; almost everything interesting is a
//! string, a number or one of the three literals. The key rule matches after
//! an object/array opener or a comma and consumes the colon, so a string value
//! is never mistaken for a key while keys on a one-line object still colour.

use crate::syntax::{HighlightKind, Language};

/// JSON.
pub static JSON: Language = Language {
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
    extra_rules: &[(
        r#"(?:^|[{,\[])[ \t]*"[^"]*"[ \t]*:"#,
        HighlightKind::Attribute,
    )],
    prose: false,
    snippets: &[],
};
