use std::io;
use std::io::Write;

use filesystem::FileSystem;
use log::{debug, info, trace, warn};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::{clear, cursor};

use crate::cmd_input::token::Token;
use crate::cmd_input::TabHandler;

macro_rules! format_u8 {
    ($($arg:tt)*) => {{
        format!($($arg)*).as_bytes()
    }};
}

pub trait DetectCursorPosAlias {
    fn get_cursor_pos(&mut self) -> (usize, usize);
}

impl<W: Write + DetectCursorPos> DetectCursorPosAlias for W {
    fn get_cursor_pos(&mut self) -> (usize, usize) {
        let pos = self.cursor_pos().expect("Unable to detect cursor position");
        (pos.0 as usize, pos.1 as usize)
    }
}

pub trait IoWriteAlias {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn flush(&mut self) -> io::Result<()>;
}

impl<W: Write + DetectCursorPos> IoWriteAlias for W {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush()
    }
}

pub struct CmdInput {
    input: Vec<char>,
    index: usize,
    previous_render: Vec<char>,
    prev_cursor_pos_x: usize,
    last_key_was_motion: bool,

    tab_handler: TabHandler,

    last_key_was_tab: bool,
}

#[inline]
fn goto_pos<T>(out: &mut T, x: usize, y: usize) -> io::Result<()>
where
    T: IoWriteAlias + DetectCursorPosAlias,
{
    out.write(cursor::Goto(x as u16, y as u16).to_string().as_bytes())?;
    Ok(())
}

impl CmdInput {
    pub fn new<T: FileSystem + 'static>(fs: T) -> CmdInput {
        CmdInput {
            input: vec![],
            index: 0,
            previous_render: vec![],
            prev_cursor_pos_x: 0,
            last_key_was_motion: false,

            tab_handler:      TabHandler::new(fs),
            last_key_was_tab: false,
        }
    }

    pub fn get_input(&self) -> &Vec<char> {
        &self.input
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_line(&self) -> (&Vec<char>, usize) {
        (&self.input, self.index)
    }

    pub fn render_line<U>(&mut self, out: &mut U, prompt_len: usize) -> io::Result<()>
    where
        U: IoWriteAlias + DetectCursorPosAlias,
    {
        let mut buf = vec![];
        let cursor_pos = out.get_cursor_pos();
        buf.reserve(self.input.len() + 10);
        buf.extend_from_slice(format_u8!(
            "{}{}{}",
            cursor::Hide,
            cursor::Goto(prompt_len as u16 + 1, cursor_pos.1 as u16),
            clear::AfterCursor,
        ));
        buf.extend_from_slice(self.input.iter().map(|x| *x as u8).collect::<Vec<u8>>().as_slice());
        buf.extend_from_slice(format_u8!(
            "{}{}{}",
            cursor::Goto((prompt_len + self.index + 1) as u16, cursor_pos.1 as u16),
            cursor::Show,
            cursor::Goto((prompt_len + self.index + 1) as u16, cursor_pos.1 as u16),
        ));

        out.write(&buf)?;
        Ok(())
    }

    pub fn insert(&mut self, key: Key) {
        debug!("Inserting key '{:?}'", key);
        match key {
            Key::Char('\t') => {
                // self.index reflects the space that's added at the end of the input sequence
                let idx_corrected = self.index - 1;
                let mut tokens = Token::parse_input(&self.input);
                let active_token = tokens
                    .iter_mut()
                    .find(|t| t.get_end_pos() <= idx_corrected && t.get_end_pos() >= idx_corrected);

                trace!(
                    "Found active token: {:?}, idx_corrected: {}",
                    active_token,
                    idx_corrected
                );

                if let Some(token) = active_token {
                    if !self.last_key_was_tab {
                        self.tab_handler.refresh();
                    }
                    if let Some(suggestion) = self.tab_handler.get_suggestion(&token.get_contents().to_string()) {
                        info!("Found suggestion: '{}'", suggestion);
                        token.set_contents(suggestion);
                        self.index = token.get_end_pos();
                        self.input = Token::assemble_tokens(&tokens);
                    }
                    else {
                        warn!("Unable to find suggestion");
                    }
                }
                self.last_key_was_tab = true;
            }

            Key::Char(c) => {
                self.input.insert(self.index, c);
                self.index += 1;
                if self.index > self.input.len() {
                    self.input.push(' ');
                }
                self.last_key_was_motion = false;
                self.last_key_was_tab = false;
            }
            Key::Backspace => {
                if self.index > 0 {
                    self.index -= 1;
                    self.input.remove(self.index);
                }
                self.last_key_was_motion = false;
                self.last_key_was_tab = false;
            }
            Key::Left => {
                if self.index != 0 {
                    self.index -= 1;
                }
                self.last_key_was_motion = true;
            }
            Key::Right => {
                if self.index != self.input.len() {
                    self.index += 1;
                }
                self.last_key_was_motion = true;
            }
            _ => {}
        }
        debug!("Input after processing key: {:?}", self.input);
    }

    pub fn clear(&mut self) {
        debug!("Clearing input");
        self.input.clear();
        self.index = 0;
    }

    pub fn get_cmd(&self) -> Vec<String> {
        debug!("Assembling tokens");
        Token::parse_input(&self.input)
            .into_iter()
            .map(|t| t.contents)
            .collect()
    }
}
