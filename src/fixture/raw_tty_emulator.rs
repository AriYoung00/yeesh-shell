use std::io;
use std::io::ErrorKind;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::slice::Iter;
use std::str::FromStr;

use crate::cmd_input::{DetectCursorPosAlias, IoWriteAlias};

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

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
struct CursorPos(pub usize, pub usize);
impl Add<(usize, usize)> for CursorPos {
    type Output = CursorPos;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        CursorPos(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl AddAssign<(usize, usize)> for CursorPos {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}
impl Sub<(usize, usize)> for CursorPos {
    type Output = CursorPos;

    fn sub(self, rhs: (usize, usize)) -> Self::Output {
        let new_0 = if rhs.0 <= self.0 { self.0 - rhs.0 } else { 0 };
        let new_1 = if rhs.1 <= self.1 { self.1 - rhs.1 } else { 0 };
        CursorPos(new_0, new_1)
    }
}
impl SubAssign<(usize, usize)> for CursorPos {
    fn sub_assign(&mut self, rhs: (usize, usize)) {
        self.0 -= if rhs.0 <= self.0 { rhs.0 } else { self.0 };
        self.1 -= if rhs.1 <= self.1 { rhs.1 } else { self.1 };
    }
}

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct RawTTYEmulator {
    text:       Vec<Vec<char>>,
    cursor_pos: CursorPos,
}

impl DetectCursorPosAlias for RawTTYEmulator {
    fn get_cursor_pos(&mut self) -> (usize, usize) {
        // tty is 1-indexed
        (self.cursor_pos.0 + 1_usize, self.cursor_pos.1 + 1_usize)
    }
}

impl RawTTYEmulator {
    pub fn new() -> RawTTYEmulator {
        RawTTYEmulator {
            text:       vec![vec![]],
            cursor_pos: CursorPos(0, 0),
        }
    }

    pub fn get_text(&self) -> &Vec<Vec<char>> {
        &self.text
    }

    pub fn get_line(&self) -> &Vec<char> {
        &self.text[self.cursor_pos.1]
    }

    pub fn get_line_str(&self) -> String {
        String::from_iter(self.text[self.cursor_pos.1].iter())
    }

    fn parse_numbers(input: &Vec<char>) -> Result<Vec<i64>, &'static str> {
        if !input.is_empty() {
            let result = Vec::from_iter(String::from_iter(input).split(";").map(|x| {
                i64::from_str(x).expect(format!("Unable to parse expected numeric string: \"{}\"", x).as_str())
            }));
            Ok(result)
        }
        else {
            Ok(vec![])
        }
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
            let c_chr = *c as char;
            if c_chr == '[' {
                is_gathering_for_num = true;
            }
            else if is_gathering_for_num && (('0' <= c_chr && c_chr <= '9') || c_chr == ';') {
                num_buf.push(c_chr);
            }
            else {
                seq_type = EscapeType::from(*c);
                break;
            }
        }

        let nums = Self::parse_numbers(&num_buf)?;
        let (arg_x, arg_y) = match nums.len() {
            0 => (1, 1),
            1 => (nums[0] as usize, nums[0] as usize),
            _ => (nums[1] as usize, nums[0] as usize),
        };

        match seq_type {
            EscapeType::LEFT => self.cursor_pos -= (arg_x, 0_usize),
            EscapeType::RIGHT => self.cursor_pos += (arg_x, 0),
            EscapeType::UP => self.cursor_pos -= (0, arg_y),
            EscapeType::DOWN => self.cursor_pos += (0, arg_y),
            // we subtract (1, 1) to account for the fact that cursor::Goto is 1-indexed
            EscapeType::HOME => self.cursor_pos = CursorPos(arg_x - 1, arg_y - 1),
            EscapeType::UNKNOWN => return Err("unknown escape sequence"),
        };
        self.extend_to_match_pos();

        Ok(())
    }
}

impl IoWriteAlias for RawTTYEmulator {
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
                    match self.handle_escape(&mut buf_iter) {
                        Err(_) => return Err(io::Error::new(ErrorKind::Other, "Unable to parse escape sequence")),
                        _ => {}
                    };
                }
                _ => {
                    if self.cursor_pos.0 < self.text[self.cursor_pos.1].len() {
                        self.text[self.cursor_pos.1][self.cursor_pos.0] = *c as char;
                    }
                    else {
                        self.text[self.cursor_pos.1].push(*c as char);
                    }
                    self.cursor_pos.0 += 1;
                }
            }
        }
        self.extend_to_match_pos();
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
