//! Shell language definition.
//!
//! Shell syntax is driven by `$` expansions rather than reserved words alone.
//! The variable rules come before the generic string rule, but the engine still
//! takes the leftmost match, so a variable inside double quotes stays part of
//! that string; only an unquoted `$HOME` is highlighted as a constant.

use crate::syntax::{HighlightKind, Language};

/// Shell (POSIX sh, bash, zsh, fish and ksh).
pub static SHELL: Language = Language {
    name: "shell",
    extensions: &["sh", "bash", "zsh", "fish", "ksh"],
    filenames: &[
        ".bashrc",
        ".bash_profile",
        ".zshrc",
        ".profile",
        ".bash_aliases",
    ],
    keywords: &[
        "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
        "in", "function", "select", "time", "return", "break", "continue", "local", "export",
        "readonly", "declare", "unset",
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
        (
            r"^[ \t]*(?:function[ \t]+)?[A-Za-z_][A-Za-z0-9_-]*[ \t]*\(\)",
            HighlightKind::Function,
        ),
    ],
    prose: false,
    snippets: &[],
};
