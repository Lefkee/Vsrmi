use crate::editor::buffer::Buffer;

/// Searches the buffer for a completion suffix for the word currently being
/// typed before the cursor.
///
/// Returns the text that should be appended so the word is finished. The search
/// priority is: language snippets → keywords → types → constants → any word
/// already present in the surrounding document.
pub fn active_snippet(buffer: &Buffer) -> Option<String> {
    let head = buffer.cursor().head;
    let line = buffer.document.line_string(head.line);

    // Never complete when the cursor is inside an existing word.
    if line.chars().nth(head.col).is_some_and(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    // Collect the characters that form the word immediately to the left.
    let prefix_chars: Vec<char> = line.chars().take(head.col).collect();
    let word_start = prefix_chars
        .iter()
        .rposition(|c| !c.is_alphanumeric() && *c != '_')
        .map_or(0, |p| p + 1);

    if word_start == prefix_chars.len() {
        return None; // cursor is not at the end of a word
    }

    let current_word: String = prefix_chars[word_start..].iter().collect();

    // Helper that returns the suffix of `candidate` after stripping `current_word`.
    let suffix = |candidate: &str| -> Option<String> {
        candidate
            .strip_prefix(current_word.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };

    // --- Language-aware completions ---
    if let Some(lang) = buffer.syntax.language() {
        // Snippets (trigger → expanded text).
        for &(trigger, text) in lang.snippets {
            if trigger.starts_with(current_word.as_str())
                && let Some(s) = suffix(text) {
                    return Some(s);
                }
        }
        // Keywords, types, constants.
        for list in [lang.keywords, lang.types, lang.constants] {
            for &word in list {
                if let Some(s) = suffix(word) {
                    return Some(s);
                }
            }
        }
    }

    // --- Local document scan (nearest match wins) ---
    let window_start = head.line.saturating_sub(1000);
    let window_end = (head.line + 1000).min(buffer.document.last_line());

    (window_start..=window_end)
        .filter_map(|line_idx| {
            let dist = head.line.abs_diff(line_idx);
            let doc_line = buffer.document.line_string(line_idx);
            // Pick the first matching token on this line; distance is the key.
            doc_line
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter_map(&suffix)
                .next()
                .map(|s| (dist, s))
        })
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, s)| s)
}
