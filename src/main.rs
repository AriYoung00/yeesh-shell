#![feature(let_chains)]

extern crate core;

mod cmd_input;
mod fixture;

use std::io;
use std::io::{stdin, stdout, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, ExitStatus};

use termion::color;
use termion::cursor;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::cmd_input::CmdInput;

fn dispatch_command(cmd_args: Vec<String>) -> io::Result<Child> {
    if cmd_args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Empty command"));
    }

    Command::new(&cmd_args[0])
        .args(if cmd_args.len() > 1 { &cmd_args[1..] } else { &[] })
        .spawn()
}

fn print_prompt<T>(status: &ExitStatus, out: &mut T) -> io::Result<usize>
where
    T: Write,
{
    const PROMPT: &str = "> ";
    if status.success() {
        write!(out, "{}", color::Fg(color::Green))?;
    }
    else {
        write!(out, "{}", color::Fg(color::Red))?;
    }

    let mut stdout_lock = stdout().lock();
    let current_pos = stdout_lock.cursor_pos().unwrap();

    write!(out, "{}", cursor::Goto(1, current_pos.1 + 1))?;
    write!(out, "{}", PROMPT)?;
    write!(out, "{}", color::Fg(color::Reset))?;
    Ok(PROMPT.len())
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    // write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "\r\n{}Hello, world!\r\n", color::Fg(color::Red)).unwrap();

    let mut cmd_input = CmdInput::new();
    let mut status = ExitStatus::from_raw(0);
    let mut prompt_len: usize = print_prompt(&status, &mut stdout).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        if let Ok(val) = c {
            match val {
                Key::Char('\n') => {
                    write!(stdout, "\r\n").unwrap();
                    let cmd_args = cmd_input.get_cmd();
                    if !cmd_args.is_empty() {
                        if cmd_args[0] == "exit" {
                            break;
                        }

                        stdout.suspend_raw_mode().unwrap();
                        if let Ok(mut child) = dispatch_command(cmd_args)
                            && let Ok(exit_status) = child.wait() {
                            status = exit_status;
                        }
                        else {
                            status = ExitStatus::from_raw(1);
                        }
                        stdout.activate_raw_mode().unwrap();
                        cmd_input.clear();
                        prompt_len = print_prompt(&status, &mut stdout).unwrap();
                    }
                }
                _ => {
                    cmd_input.insert(val);
                    cmd_input.render_line(&mut stdout, prompt_len).unwrap();
                }
            }
        }

        stdout.flush().unwrap();
    }
}
