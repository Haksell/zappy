use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::history::DefaultHistory;
use rustyline::{ColorMode, Completer, Editor, Helper, Hinter, Validator};
use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};

pub struct LineReader {
    rl: Editor<MaskingHighlighter, DefaultHistory>,
    prompt: String,
}

impl LineReader {
    pub fn new(prompt: String) -> Result<Self, ReadlineError> {
        let h = MaskingHighlighter { masking: false };
        let mut rl = Editor::new()?;
        rl.set_helper(Some(h));
        Ok(Self { rl, prompt })
    }

    pub fn readline(&mut self) -> Result<String, ReadlineError> {
        self.rl.readline(&self.prompt)
    }

    pub fn readline_prompt(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.rl.readline(prompt)
    }

    fn mask_input(&mut self) {
        self.rl.helper_mut().expect("No helper").masking = true;
        self.rl.set_color_mode(ColorMode::Forced);
        self.rl.set_auto_add_history(false);
    }

    fn unmask_input(&mut self) {
        self.rl.helper_mut().expect("No helper").masking = false;
        self.rl.set_color_mode(ColorMode::Enabled);
        self.rl.set_auto_add_history(true);
    }

    pub fn read_secret(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.mask_input();
        let mut guard = self.rl.set_cursor_visibility(false)?;

        let result = self.readline_prompt(prompt);
        self.unmask_input();
        guard.take();

        result
    }
}

#[derive(Completer, Helper, Hinter, Validator)]
struct MaskingHighlighter {
    masking: bool,
}

impl Highlighter for MaskingHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        use unicode_width::UnicodeWidthStr;
        if self.masking {
            Owned("*".repeat(line.width()))
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, kind: CmdKind) -> bool {
        match kind {
            CmdKind::MoveCursor => false,
            _ => self.masking,
        }
    }
}
