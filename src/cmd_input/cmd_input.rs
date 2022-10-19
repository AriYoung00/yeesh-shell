use std::io;
use std::io::Write;

use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::{clear, cursor};
use crate::cmd_input::TabHandler;
use crate::cmd_input::token::Token;

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
    pub fn new() -> CmdInput {
        CmdInput {
            input: vec![],
            index: 0,
            previous_render: vec![],
            prev_cursor_pos_x: 0,
            last_key_was_motion: false,
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

    pub fn render_line<T>(&mut self, out: &mut T, prompt_len: usize) -> io::Result<()>
    where
        T: IoWriteAlias + DetectCursorPosAlias,
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
        match key {
            Key::Char(c) => {
                self.input.insert(self.index, c);
                self.index += 1;
                if self.index > self.input.len() {
                    self.input.push(' ');
                }
                self.last_key_was_motion = false;
            }
            Key::Backspace => {
                if self.index > 0 {
                    self.index -= 1;
                    self.input.remove(self.index);
                }
                self.last_key_was_motion = false;
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
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.index = 0;
    }

    pub fn get_cmd(&self) -> Vec<String> {
        let mut cmd_args = vec![];
        let mut current_arg = vec![];
        let mut is_quoted = false;
        let mut quote_char = '\'';

        // TODO: match quotes -- this would currently be a valid quoted string: "hello'
        for c in self.input.iter() {
            match c {
                ' ' if !is_quoted => {
                    if self.input.len() > 1 {
                        cmd_args.push(String::from_iter(current_arg.iter()));
                    }
                    current_arg.clear();
                }
                '"' | '\'' if !is_quoted => {
                    is_quoted = true;
                    quote_char = *c;
                }
                '"' | '\'' if is_quoted && *c == quote_char => {
                    is_quoted = false;
                }
                _ => current_arg.push(*c),
            }
        }

        if self.input.len() > 1 {
            cmd_args.push(String::from_iter(current_arg.iter()));
        }
        cmd_args
    }
}
