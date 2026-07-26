//! # Buffer editing
//!
//! **Purpose:** the operations that change text.
//!
//! **Responsibility:** apply an edit at every cursor, record it in the buffer's
//! [`History`](crate::undo::History), and leave the cursors somewhere sensible.
//! Splitting this out of `buffer` keeps that module about *structure* and this
//! one about *change*.
//!
//! Every operation walks the cursors back to front. Editing the last cursor
//! first means the offsets of all the earlier ones are still valid when their
//! turn comes, which removes the whole class of "fix up the other cursors after
//! the edit" bugs.
//!
//! **Public API:** the `impl Buffer` block below.

use super::Buffer;
use crate::config::Config;
use crate::editor::cursor::Position;
use crate::editor::document::indent;
use crate::editor::selection::Range;
use crate::undo::Change;

impl Buffer {
    /// Insert `text` at every cursor.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let length = text.chars().count();
        for index in self.edit_order() {
            let before = self.cursors[index].head;
            let at = self.document.pos_to_char(before);

            self.document.insert(at, text);
            let after = self.document.char_to_pos(at + length);
            self.history
                .record(Change::insertion(at, text), before, after);
            self.cursors[index].move_to(after, false);
        }
        self.invalidate_syntax_from(self.cursors[0].head.line);
        self.resort();
    }

    /// Break the line at every cursor, carrying the indentation over.
    pub fn insert_newline(&mut self, config: &Config) {
        for index in self.edit_order() {
            let before = self.cursors[index].head;
            let at = self.document.pos_to_char(before);

            let mut text = String::from("\n");
            if config.auto_indent {
                text.push_str(&indent::auto_indent_for_new_line(
                    &self.document,
                    before.line,
                    before.col,
                    config.tab_width,
                    config.expand_tabs,
                ));
            }

            self.document.insert(at, &text);
            let after = self.document.char_to_pos(at + text.chars().count());
            self.history
                .record(Change::insertion(at, text.as_str()), before, after);
            self.cursors[index].move_to(after, false);
        }
        self.invalidate_syntax_from(self.cursors[0].head.line);
        self.resort();
    }

    /// Insert one indentation step at every cursor.
    ///
    /// With `expand_tabs` the step is only as wide as it needs to be to reach
    /// the next tab stop, so pressing Tab mid-line lines up instead of always
    /// jumping a full `tab_width`.
    pub fn insert_indent(&mut self, config: &Config) {
        if !config.expand_tabs {
            self.insert_text("\t");
            return;
        }
        for index in self.edit_order() {
            let before = self.cursors[index].head;
            let width = config.tab_width - (before.col % config.tab_width);
            let text = " ".repeat(width);
            let at = self.document.pos_to_char(before);

            self.document.insert(at, &text);
            let after = self.document.char_to_pos(at + width);
            self.history
                .record(Change::insertion(at, text.as_str()), before, after);
            self.cursors[index].move_to(after, false);
        }
        self.invalidate_syntax_from(self.cursors[0].head.line);
        self.resort();
    }

    /// Delete the character before every cursor.
    ///
    /// Inside leading whitespace this removes a whole indentation step, which is
    /// what makes backspace feel symmetric with Tab.
    pub fn delete_backward(&mut self, config: &Config) {
        for index in self.edit_order() {
            let before = self.cursors[index].head;
            let at = self.document.pos_to_char(before);
            if at == 0 {
                continue;
            }
            let start = at - self.backspace_width(before, config);

            let removed = self.document.remove(start, at);
            let after = self.document.char_to_pos(start);
            self.history
                .record(Change::deletion(start, removed), before, after);
            self.cursors[index].move_to(after, false);
        }
        self.invalidate_syntax_from(self.cursors[0].head.line);
        self.resort();
    }

    /// Delete the character under every cursor.
    pub fn delete_forward(&mut self) {
        for index in self.edit_order() {
            let before = self.cursors[index].head;
            let at = self.document.pos_to_char(before);
            if at >= self.document.len_chars() {
                continue;
            }

            let removed = self.document.remove(at, at + 1);
            let after = self.document.char_to_pos(at);
            self.history
                .record(Change::deletion(at, removed), before, after);
            self.cursors[index].move_to(after, false);
        }
        self.invalidate_syntax_from(self.cursors[0].head.line);
        self.resort();
    }

    /// Delete a span and collapse the primary cursor onto its start.
    ///
    /// Returns the removed text so callers can put it on the clipboard.
    pub fn delete_range(&mut self, range: Range) -> String {
        if range.is_empty() {
            return String::new();
        }
        let before = self.cursor().head;
        let removed = self.document.remove(range.start, range.end);
        let after = self.document.char_to_pos(range.start);

        self.history.record(
            Change::deletion(range.start, removed.clone()),
            before,
            after,
        );
        self.clear_secondary_cursors();
        self.cursor_mut().move_to(after, false);
        self.invalidate_syntax_from(after.line);
        self.history.checkpoint();
        removed
    }

    /// Insert clipboard content at the primary cursor.
    ///
    /// Line-wise content goes on its own line *below* the caret, the way `p`
    /// behaves in vi; a fragment is spliced in at the caret. Getting this wrong
    /// is the difference between pasting a function after the current one and
    /// pasting it into the middle of a line.
    pub fn paste(&mut self, text: &str, line_wise: bool) {
        if text.is_empty() {
            return;
        }
        if !line_wise {
            self.insert_text(text);
            self.checkpoint();
            return;
        }

        let before = self.cursor().head;
        let next_line = before.line + 1;
        let (at, payload) = if next_line <= self.document.last_line() {
            let mut payload = text.to_string();
            if !payload.ends_with('\n') {
                payload.push('\n');
            }
            (self.document.line_start(next_line), payload)
        } else {
            // Nothing follows this line, so the payload has to bring its own
            // leading newline instead of a trailing one.
            let payload = format!("\n{}", text.trim_end_matches('\n'));
            (self.document.len_chars(), payload)
        };

        self.document.insert(at, &payload);
        let landed = self
            .document
            .char_to_pos(at + usize::from(payload.starts_with('\n')));
        self.history
            .record(Change::insertion(at, payload.as_str()), before, landed);

        self.clear_secondary_cursors();
        self.cursor_mut().move_to(landed, false);
        self.invalidate_syntax_from(before.line);
        self.checkpoint();
    }

    /// Strip trailing spaces and tabs from every line.
    ///
    /// Returns how many lines changed. Runs on save, back to front so earlier
    /// offsets stay valid, and as a single undo step so one `u` puts it all back.
    pub fn trim_trailing_whitespace(&mut self) -> usize {
        let before = self.cursor().head;
        let mut trimmed = 0;

        self.history.checkpoint();
        for line in (0..self.document.len_lines()).rev() {
            let text = self.document.line_string(line);
            let kept = text.trim_end_matches([' ', '\t']).chars().count();
            let length = text.chars().count();
            if kept == length {
                continue;
            }
            let start = self.document.line_start(line) + kept;
            let removed = self.document.remove(start, start + (length - kept));
            self.history.record(
                Change::deletion(start, removed),
                before,
                self.document.char_to_pos(start),
            );
            trimmed += 1;
        }

        if trimmed > 0 {
            self.invalidate_syntax_from(0);
            self.clamp_cursors(true);
            self.history.checkpoint();
        }
        trimmed
    }

    /// Undo one step, moving the caret to where the edit started.
    ///
    /// Returns `false` when there is nothing left to undo.
    pub fn undo(&mut self) -> bool {
        let Some(position) = self.history.undo(&mut self.document) else {
            return false;
        };
        self.restore_caret(position);
        true
    }

    /// Redo one step.
    ///
    /// Returns `false` when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        let Some(position) = self.history.redo(&mut self.document) else {
            return false;
        };
        self.restore_caret(position);
        true
    }

    /// End the current undo step, so the next edit starts a new one.
    pub fn checkpoint(&mut self) {
        self.history.checkpoint();
    }

    /// How many characters backspace should remove at `position`.
    fn backspace_width(&self, position: Position, config: &Config) -> usize {
        if !config.expand_tabs || position.col == 0 {
            return 1;
        }
        // Only collapse a full indent step when everything to the left is
        // spaces; otherwise backspace inside a word would eat several letters.
        let leading_spaces = self
            .document
            .line(position.line)
            .chars()
            .take(position.col)
            .all(|ch| ch == ' ');
        if !leading_spaces {
            return 1;
        }
        let step = (position.col - 1) % config.tab_width + 1;
        step.min(position.col)
    }

    /// Put a single cursor at `position` after a history operation.
    fn restore_caret(&mut self, position: Position) {
        self.clear_secondary_cursors();
        let clamped = self.document.clamp(position, true);
        self.cursor_mut().move_to(clamped, false);
        self.invalidate_syntax_from(clamped.line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::cursor::Cursor;
    use crate::editor::document::Document;

    fn buffer(text: &str) -> Buffer {
        Buffer::new(Document::from_text(text, None))
    }

    fn config() -> Config {
        Config::default()
    }

    fn text_of(buffer: &Buffer) -> String {
        buffer.document.text().to_string()
    }

    #[test]
    fn typing_moves_the_caret_along() {
        let mut buf = buffer("");
        buf.insert_text("hi");
        assert_eq!(text_of(&buf), "hi");
        assert_eq!(buf.cursor().head, Position::new(0, 2));
    }

    #[test]
    fn newline_carries_the_indentation_over() {
        let mut buf = buffer("    let x = 1;");
        buf.cursor_mut().move_to(Position::new(0, 14), false);
        buf.insert_newline(&config());
        assert_eq!(text_of(&buf), "    let x = 1;\n    ");
        assert_eq!(buf.cursor().head, Position::new(1, 4));
    }

    #[test]
    fn newline_after_an_opening_brace_indents_one_more_level() {
        let mut buf = buffer("fn main() {");
        buf.cursor_mut().move_to(Position::new(0, 11), false);
        buf.insert_newline(&config());
        assert_eq!(text_of(&buf), "fn main() {\n    ");
    }

    #[test]
    fn tab_aligns_to_the_next_tab_stop() {
        let mut buf = buffer("ab");
        buf.cursor_mut().move_to(Position::new(0, 2), false);
        buf.insert_indent(&config());
        assert_eq!(text_of(&buf), "ab  ");
    }

    #[test]
    fn backspace_removes_a_whole_indent_step() {
        let mut buf = buffer("        x");
        buf.cursor_mut().move_to(Position::new(0, 8), false);
        buf.delete_backward(&config());
        assert_eq!(text_of(&buf), "    x");
    }

    #[test]
    fn backspace_inside_a_word_removes_one_character() {
        let mut buf = buffer("word");
        buf.cursor_mut().move_to(Position::new(0, 4), false);
        buf.delete_backward(&config());
        assert_eq!(text_of(&buf), "wor");
    }

    #[test]
    fn backspace_at_the_start_of_a_line_joins_it_to_the_previous_one() {
        let mut buf = buffer("ab\ncd");
        buf.cursor_mut().move_to(Position::new(1, 0), false);
        buf.delete_backward(&config());
        assert_eq!(text_of(&buf), "abcd");
        assert_eq!(buf.cursor().head, Position::new(0, 2));
    }

    #[test]
    fn multiple_cursors_all_receive_the_edit() {
        let mut buf = buffer("a\na\na");
        buf.add_cursor(Cursor::at(Position::new(1, 0)));
        buf.add_cursor(Cursor::at(Position::new(2, 0)));
        buf.insert_text("x");
        assert_eq!(text_of(&buf), "xa\nxa\nxa");
        assert_eq!(buf.cursors().len(), 3);
    }

    #[test]
    fn deleting_a_range_returns_the_removed_text() {
        let mut buf = buffer("hello world");
        let removed = buf.delete_range(Range { start: 0, end: 6 });
        assert_eq!(removed, "hello ");
        assert_eq!(text_of(&buf), "world");
        assert_eq!(buf.cursor().head, Position::ZERO);
    }

    #[test]
    fn a_line_wise_paste_lands_on_its_own_line_below() {
        let mut buf = buffer("first\nsecond");
        buf.paste("copied\n", true);
        assert_eq!(text_of(&buf), "first\ncopied\nsecond");
        assert_eq!(buf.cursor().head, Position::new(1, 0));
    }

    #[test]
    fn a_line_wise_paste_on_the_last_line_appends_a_new_line() {
        let mut buf = buffer("only");
        buf.paste("copied\n", true);
        assert_eq!(text_of(&buf), "only\ncopied");
        assert_eq!(buf.cursor().head, Position::new(1, 0));
    }

    #[test]
    fn a_fragment_paste_splices_into_the_current_line() {
        let mut buf = buffer("ac");
        buf.cursor_mut().move_to(Position::new(0, 1), false);
        buf.paste("b", false);
        assert_eq!(text_of(&buf), "abc");
    }

    #[test]
    fn undo_and_redo_round_trip_an_edit() {
        let mut buf = buffer("start");
        buf.cursor_mut().move_to(Position::new(0, 5), false);
        buf.insert_text("!");
        assert!(buf.undo());
        assert_eq!(text_of(&buf), "start");
        assert!(buf.redo());
        assert_eq!(text_of(&buf), "start!");
        assert!(!buf.redo());
    }
}
