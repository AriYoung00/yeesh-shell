use core::num::dec2flt::parse::parse_number;
use std::io;
use std::io::Error;
use std::slice::Iter;
use std::str::FromStr;

use termion::cursor::DetectCursorPos;

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
struct RawTTYEmulator {
    text:       Vec<Vec<char>>,
    cursor_pos: (usize, usize),
}

enum EscapeType {
    HOME,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    UNKNOWN,
}
impl From<u8> for EscapeType {
    fn from(input: u8) -> Self {
        match input {
            b'H' => Self::HOME,
            b'A' => Self::UP,
            b'B' => Self::DOWN,
            b'C' => Self::RIGHT,
            b'D' => Self::LEFT,
            _ => Self::UNKNOWN,
        }
    }
}

impl RawTTYEmulator {
    pub fn new() -> RawTTYEmulator {
        RawTTYEmulator {
            text:       vec![vec![]],
            cursor_pos: (0, 0),
        }
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        self.cursor_pos
    }

    pub fn get_current_line(&self) -> &Vec<char> {
        &self.text[self.cursor_pos.1]
    }

    fn parse_numbers(input: Vec<&u8>) -> Result<Vec<i64>, &'static str> {
        String::from_utf8_lossy(input[..=])
            .split(";")
            .map(|x| i64::from_str(x)?)
            .collect()
    }

    fn extend_to_match_pos(&mut self) {
        while self.cursor_pos.1 >= self.text.len() {
            self.text.push(vec![])
        }
        while self.cursor_pos.0 >= self.text[self.cursor_pos.1].len() {
            self.text[self.cursor_pos.1].push(' ');
        }
    }

    fn handle_escape(&mut self, buf_iter: &mut Iter<u8>) -> Result<(), &'static str> {
        let mut num_buf = vec![];
        let mut is_gathering_for_num = false;
        let mut seq_type = EscapeType::UNKNOWN;
        while let Some(c) = buf_iter.next() {
            match *c {
                b'[' => is_gathering_for_num = true,
                _ if is_gathering_for_num => buf.push(c),
                _ => {
                    seq_type = EscapeType::from(*c);
                    break;
                }
            }
        }

        let (arg_x, arg_y) = match Self::parse_numbers(num_buf).len() {
            0 => (1, 1),
            1 => (numbers[0], numbers[0]),
            _ => (numbers[0], numbers[1]),
        };

        match seq_type {
            EscapeType::LEFT => self.cursor_pos -= (arg_x, 0),
            EscapeType::RIGHT => self.cursor_pos += (arg_x, 0),
            EscapeType::UP => self.cursor_pos -= (0, arg_y),
            EscapeType::DOWN => self.cursor_pos += (0, arg_y),
            // we subtract (1, 1) to account for the fact that cursor::Goto is 1-indexed
            EscapeType::HOME => self.cursor_pos = (arg_x, arg_y) - (1, 1),
            EscapeType::UNKNOWN => return Err("unknown escape sequence"),
        };
        self.extend_to_match_pos();

        Ok(())
    }
}

impl io::Write for RawTTYEmulator {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buf_iter = buf.iter();
        while let Some(c) = buf_iter.next() {
            match *c {
                b'\n' => {
                    if self.cursor_pos.1 == self.text.len() - 1 {
                        self.text.push(vec![]);
                    }
                    self.cursor_pos.1 += 1;
                }
                b'\r' => {
                    self.cursor_pos.0 = 0;
                }
                b'\x1B' => {
                    self.handle_escape(&mut buf_iter);
                }
                _ => {
                    self.text[self.cursor_pos.1].insert(self.cursor_pos.0, *c as char);
                    self.cursor_pos.0 += 1;
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
