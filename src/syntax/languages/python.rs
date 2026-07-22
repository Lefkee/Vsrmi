//! Python language definition.
//!
//! Python has no block comments, but triple-quoted strings behave like one for a
//! per-line scanner: they open, run over any number of lines, and close. Mapping
//! `"""` onto the block-comment machinery is what makes docstrings survive a
//! scroll, and colouring them as comments matches what they are used for.

use crate::syntax::{HighlightKind, Language};

/// Python.
pub static PYTHON: Language = Language {
    name: "python",
    extensions: &["py", "pyi", "pyw"],
    filenames: &[],
    keywords: &[
        "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def",
        "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in",
        "is", "lambda", "match", "nonlocal", "not", "or", "pass", "raise", "return", "try",
        "while", "with", "yield",
    ],
    types: &[
        "bool",
        "bytearray",
        "bytes",
        "complex",
        "dict",
        "float",
        "frozenset",
        "int",
        "list",
        "object",
        "set",
        "str",
        "tuple",
        "type",
        "Any",
        "Callable",
        "Dict",
        "Iterable",
        "Iterator",
        "List",
        "Optional",
        "Sequence",
        "Set",
        "Tuple",
        "Union",
    ],
    constants: &[
        "True",
        "False",
        "None",
        "NotImplemented",
        "Ellipsis",
        "self",
        "cls",
        "__name__",
        "__file__",
    ],
    line_comment: Some("#"),
    block_comment: Some((r#"""""#, r#"""""#)),
    nested_block_comments: false,
    macro_suffix: false,
    capitalised_types: true,
    extra_rules: &[
        (r"^\s*@[A-Za-z_][\w.]*", HighlightKind::Attribute),
        // The `'''` form of a docstring, when it opens and closes on one line.
        (r"'''[^']*'''", HighlightKind::Comment),
        // Prefixed string literals: f-strings, byte strings, raw strings.
        (
            r#"(?:[fFrRbBuU]{1,2})?"(?:[^"\\]|\\.)*"|(?:[fFrRbBuU]{1,2})?'(?:[^'\\]|\\.)*'"#,
            HighlightKind::String,
        ),
    ],
    prose: false,
};
