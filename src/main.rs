#![feature(let_chains)]
#![feature(iter_intersperse)]
#![feature(is_some_and)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(non_camel_case_types)]

mod cmd_input;
mod error;
mod fixture;
mod intrinsics;
mod prompt;

use std::env;
use std::io;
use std::io::{stderr, stdin, stdout, Stdout, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

use filesystem::OsFileSystem;
use intrinsics::find_intrinsic;
use prompt::print_prompt;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::cmd_input::CmdInput;

fn dispatch_command(cmd_args: Vec<String>) -> io::Result<Child> {
    if cmd_args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Empty command"));
    }

    Command::new(&cmd_args[0])
        .args(if cmd_args.len() > 1 { &cmd_args[1..] } else { &[] })
        .spawn()
}

fn handle_cd(cmd_args: &[String]) -> io::Result<ExitStatus> {
    if cmd_args.len() == 0 {
        let path = Path::new("~");
        env::set_current_dir(&path)?;
    }
    else {
        match cmd_args[0].chars().nth(0).unwrap() {
            '/' | '~' => env::set_current_dir(&cmd_args[0])?,
            _ => {
                let mut path = env::current_dir()?;
                path.push(cmd_args[0].as_str());
                env::set_current_dir(&path)?;
            }
        }
    }

    Ok(ExitStatus::from_raw(0))
}

fn handle_command(stdout: &mut RawTerminal<Stdout>, cmd_input: &mut CmdInput) -> Option<ExitStatus> {
    let status: ExitStatus;

    let cmd_args = cmd_input.get_cmd();
    if !cmd_args.is_empty() && let Some(intrinsic) = find_intrinsic(&cmd_args[0]) {
        if cmd_args.len() == 2 && (cmd_args[1] == "--help" || cmd_args[1] == "-h") {
            println!("{}\r", intrinsic.description);
            status = ExitStatus::from_raw(0);
        } else {
            match (intrinsic.handler)(&cmd_args[1..]) {
                Ok(output) => {
                    write!(stdout, "{}", output).unwrap();
                    status = ExitStatus::from_raw(0);
                }
                Err(err) => {
                    write!(stderr(), "{}", err).unwrap();
                    status = ExitStatus::from_raw(1);
                }
            }
        }
        if intrinsic.command == "exit" {
            return None;
        }
    }
    else if !cmd_args.is_empty() {
        stdout.suspend_raw_mode().unwrap();
        if let Ok(mut child) = dispatch_command(cmd_args)
            && let Ok(exit_status) = child.wait() {
            status = exit_status;
        } else {
            status = ExitStatus::from_raw(1);
        }
        stdout.activate_raw_mode().unwrap();
    }
    else {
        status = ExitStatus::from_raw(0);
    }

    Some(status)
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    // write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "\r\n{}Hello, world!\r\n", color::Fg(color::Red)).unwrap();

    let filesystem = OsFileSystem::new();

    let mut cmd_input = CmdInput::new(filesystem);
    let mut status = ExitStatus::from_raw(0);
    let mut prompt_len: usize = print_prompt(&status, &mut stdout).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        if let Ok(val) = c {
            match val {
                Key::Char('\n') => {
                    write!(stdout, "\r\n").unwrap();
                    if let Some(new_status) = handle_command(&mut stdout, &mut cmd_input) {
                        status = new_status;
                    }
                    else {
                        break;
                    }

                    cmd_input.clear();
                    prompt_len = print_prompt(&status, &mut stdout).unwrap();
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
