use std::io;

use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::{clear, cursor};

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
        T: io::Write,
    {
        let cursor_pos = out.cursor_pos().unwrap();
        let (cursor_pos_x, cursor_pos_y) = (cursor_pos.0 as usize, cursor_pos.1 as usize);
        let new_cursor_pos_x = if cursor_pos_x != self.index + prompt_len {
            self.index + prompt_len
        }
        else {
            cursor_pos_x
        };

        write!(out, "{}", cursor::Goto(new_cursor_pos_x as u16, cursor_pos_y as u16))?;

        if self.index > 0 {
            write!(out, "{}", String::from_iter(self.input[self.index - 1..].iter()))?;
        }
        else {
            write!(out, "{}", String::from_iter(self.input.iter()))?;
        }

        if cursor_pos_x < self.prev_cursor_pos_x {
            write!(out, "{}", cursor::Goto(cursor_pos_x as u16, cursor_pos_y as u16))?;
            write!(out, "{}", clear::AfterCursor)?;
            out.flush()?;
        }

        write!(
            out,
            "{}",
            cursor::Goto(new_cursor_pos_x as u16 + 1, cursor_pos_y as u16)
        )?;
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
