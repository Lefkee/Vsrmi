//! # Clipboard
//!
//! **Purpose:** yank and paste, with or without a desktop session.
//!
//! **Responsibility:** wrap [`arboard`] and fall back to an internal register
//! when the system clipboard is unavailable — over SSH, in a container, or when
//! the user turns it off. Every copy also lands in the internal register, so
//! paste keeps working even if the system clipboard fails between the two.
//!
//! Whether a yank was *line-wise* travels with the text: pasting a whole line
//! should put it on its own line, while pasting a fragment should splice it in
//! where the caret is. The system clipboard cannot carry that flag, so it is
//! tracked here.
//!
//! **Public API:** [`Clipboard`].

/// The editor's copy buffer.
pub struct Clipboard {
    system: Option<arboard::Clipboard>,
    register: String,
    line_wise: bool,
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard")
            .field("system", &self.system.is_some())
            .field("register", &self.register.len())
            .field("line_wise", &self.line_wise)
            .finish()
    }
}

impl Clipboard {
    /// Connect to the system clipboard when asked, and when one exists.
    #[must_use]
    pub fn new(use_system: bool) -> Self {
        Self {
            system: use_system.then(|| arboard::Clipboard::new().ok()).flatten(),
            register: String::new(),
            line_wise: false,
        }
    }

    /// Whether the system clipboard is in use, as opposed to the internal one.
    #[must_use]
    pub const fn uses_system(&self) -> bool {
        self.system.is_some()
    }

    /// Store `text`, remembering whether it covers whole lines.
    pub fn set(&mut self, text: String, line_wise: bool) {
        if let Some(system) = self.system.as_mut() {
            // A clipboard failure must not lose the yank, so ignore the error
            // and rely on the register below.
            let _ = system.set_text(text.clone());
        }
        self.register = text;
        self.line_wise = line_wise;
    }

    /// Retrieve the clipboard contents.
    ///
    /// Prefers the system clipboard so that copying from another application
    /// works, but falls back to the register when it is empty or unreadable.
    pub fn get(&mut self) -> String {
        if let Some(system) = self.system.as_mut()
            && let Ok(text) = system.get_text()
            && !text.is_empty()
        {
            // Text from elsewhere has no line-wise flag; treat it as a fragment
            // unless it is exactly what we put there.
            if text != self.register {
                self.line_wise = text.ends_with('\n');
            }
            return text;
        }
        self.register.clone()
    }

    /// Whether the stored text was yanked as whole lines.
    #[must_use]
    pub const fn is_line_wise(&self) -> bool {
        self.line_wise
    }
}
