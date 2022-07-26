use std::io;
use std::io::Write;

use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::{clear, cursor};

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
    prev_cursor_pos_x: usize,
}

impl CmdInput {
    pub fn new() -> CmdInput {
        CmdInput {
            input: vec![],
            index: 0,
            prev_cursor_pos_x: 0,
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
        let (cursor_pos_x, cursor_pos_y) = out.get_cursor_pos();
        let new_cursor_pos_x = if cursor_pos_x != self.index + prompt_len {
            self.index + prompt_len
        }
        else {
            cursor_pos_x
        };

        out.write(format_u8!(
            "{}",
            cursor::Goto(new_cursor_pos_x as u16, cursor_pos_y as u16)
        ))?;

        if self.index > 0 {
            out.write(format_u8!("{}", String::from_iter(self.input[self.index - 1..].iter())))?;
        }
        else {
            out.write(format_u8!("{}", String::from_iter(self.input.iter())))?;
        }

        if cursor_pos_x < self.prev_cursor_pos_x {
            out.write(format_u8!("{}", cursor::Goto(cursor_pos_x as u16, cursor_pos_y as u16)))?;
            out.write(format_u8!("{}", clear::AfterCursor))?;
        }

        out.write(format_u8!(
            "{}",
            cursor::Goto(new_cursor_pos_x as u16 + 1, cursor_pos_y as u16)
        ))?;
        self.prev_cursor_pos_x = new_cursor_pos_x + 1;
        Ok(())
    }

    pub fn insert(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                self.input.insert(self.index, c);
                self.index += 1;
                // print!("{}", c);
            }
            Key::Backspace => {
                if self.index != 0 {
                    self.index -= 1;
                    self.input.remove(self.index);
                }
            }
            Key::Left => {
                if self.index != 0 {
                    self.index -= 1;
                }
            }
            Key::Right => {
                if self.index != self.input.len() {
                    self.index += 1;
                }
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

        for c in self.input.iter() {
            match c {
                ' ' if !is_quoted => {
                    cmd_args.push(String::from_iter(current_arg.iter()));
                    current_arg.clear();
                }
                '"' | '\'' => is_quoted = !is_quoted,
                _ => current_arg.push(*c),
            }
        }

        cmd_args.push(String::from_iter(current_arg.iter()));
        cmd_args
    }
}
