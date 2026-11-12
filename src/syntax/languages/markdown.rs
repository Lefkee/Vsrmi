//! Markdown language definition.
//!
//! Markdown is `prose`, so the generic identifier, operator and punctuation
//! rules are switched off — running them over English turns every full stop and
//! bracket into a coloured token and makes the text harder to read, not easier.
//!
//! Fenced code blocks are mapped onto the block-comment machinery: they open and
//! close with the same token and run over many lines, which is exactly the shape
//! that machinery handles.

use crate::syntax::{HighlightKind, Language};

/// Markdown.
pub static MARKDOWN: Language = Language {
    name: "markdown",
    extensions: &["md", "markdown", "mdown", "mkd"],
    filenames: &["readme", "changelog"],
    keywords: &[],
    types: &[],
    constants: &[],
    line_comment: None,
    block_comment: Some(("```", "```")),
    nested_block_comments: false,
    macro_suffix: false,
    capitalised_types: false,
    extra_rules: &[
        (r"^#{1,6}\s.*", HighlightKind::Heading),
        // Setext headings and horizontal rules.
        (r"^(?:={3,}|-{3,}|\*{3,})\s*$", HighlightKind::Heading),
        (r"^>\s?.*", HighlightKind::Comment),
        (r"`[^`]+`", HighlightKind::String),
        (r"\*\*[^*]+\*\*|__[^_]+__", HighlightKind::Emphasis),
        (r"\*[^*\s][^*]*\*|_[^_\s][^_]*_", HighlightKind::Emphasis),
        (r"!?\[[^\]]*\]\([^)]*\)", HighlightKind::Link),
        (r"<?https?://[^\s>)]+>?", HighlightKind::Link),
        // List markers and task boxes, at the start of a line only.
        (r"^\s*(?:[-*+]|\d+[.)])\s", HighlightKind::Punctuation),
        (r"^\s*(?:[-*+])\s\[[ xX]\]", HighlightKind::Constant),
    ],
    prose: true,
    snippets: &[],
};
