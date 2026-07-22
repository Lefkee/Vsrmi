//! Rust language definition.
//!
//! The extra rules exist because the generic scanner would get three things
//! wrong: attributes look like comments-that-are-not, raw strings ignore
//! backslash escapes, and a lifetime `'a` looks like the start of a character
//! literal. Putting the character-literal pattern *before* the lifetime pattern
//! is what disambiguates `'a'` from `'a`.

use crate::syntax::{HighlightKind, Language};

/// Rust.
pub static RUST: Language = Language {
    name: "rust",
    extensions: &["rs"],
    filenames: &[],
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "static", "struct", "super", "trait", "type", "union",
        "unsafe", "use", "where", "while", "yield",
    ],
    types: &[
        "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
        "u16", "u32", "u64", "u128", "usize", "String", "Vec", "Option", "Result", "Box", "Rc",
        "Arc", "Cow", "HashMap", "HashSet", "BTreeMap", "VecDeque", "Self",
    ],
    constants: &["true", "false", "None", "Some", "Ok", "Err"],
    line_comment: Some("//"),
    block_comment: Some(("/*", "*/")),
    nested_block_comments: true,
    macro_suffix: true,
    capitalised_types: true,
    extra_rules: &[
        // Doc comments before ordinary ones, so they can be told apart.
        (r"///.*|//!.*", HighlightKind::Comment),
        (r"#!?\[[^\]]*\]?", HighlightKind::Attribute),
        // Raw strings: no escape processing, any number of hashes. The pattern
        // itself contains `"#`, so it needs the `##` string delimiters.
        (r##"r#*"[^"]*"#*"##, HighlightKind::String),
        (r#"b"(?:[^"\\]|\\.)*""#, HighlightKind::String),
        // Character literal, ahead of the lifetime rule below.
        (r"b?'(?:[^'\\]|\\.)'", HighlightKind::String),
        (r"'(?:static|_|[a-z][a-z0-9_]*)", HighlightKind::Type),
    ],
    prose: false,
};
