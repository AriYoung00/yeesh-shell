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

use std::io;
use std::io::{stderr, stdin, stdout, Stdout, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, ExitStatus};

use filesystem::OsFileSystem;
use intrinsics::find_intrinsic;
use log::info;
use prompt::print_prompt;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::cmd_input::CmdInput;
use crate::HandleKeyResult::{CommandStatus, Continue, Exit};

fn dispatch_command(cmd_args: Vec<String>) -> io::Result<Child> {
    if cmd_args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Empty command"));
    }

    Command::new(&cmd_args[0])
        .args(if cmd_args.len() > 1 { &cmd_args[1..] } else { &[] })
        .spawn()
}

fn handle_command(stdout: &mut RawTerminal<Stdout>, cmd_input: &mut CmdInput) -> Option<ExitStatus> {
    let status: ExitStatus;

    let cmd_args = cmd_input.get_cmd();
    if !cmd_args.is_empty() && let Some(intrinsic) = find_intrinsic(&cmd_args[0]) {
        if cmd_args.len() == 2 && (cmd_args[1] == "--help" || cmd_args[1] == "-h") {
            println!("{}\r", intrinsic.get_description());
            status = ExitStatus::from_raw(0);
        } else {
            match intrinsic.handler(&cmd_args[1..]) {
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
        if intrinsic.get_command() == "exit" {
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

enum HandleKeyResult {
    Continue,
    CommandStatus(ExitStatus),
    Exit,
}

fn handle_key(
    mut stdout: &mut RawTerminal<Stdout>,
    mut cmd_input: &mut CmdInput,
    prompt_len: usize,
    val: Key,
) -> HandleKeyResult {
    match val {
        Key::Char('\n') => {
            write!(stdout, "\r\n").unwrap();
            let rval = if let Some(new_status) = handle_command(&mut stdout, &mut cmd_input) {
                CommandStatus(new_status)
            }
            else {
                Exit
            };

            rval
        }
        _ => {
            cmd_input.insert(val);
            cmd_input.render_line(&mut stdout, prompt_len).unwrap();
            Continue
        }
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    // write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "\r\n{}Hello, world!\r\n", color::Fg(color::Red)).unwrap();

    let config_str = include_str!("logger_config.yaml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();
    let filesystem = OsFileSystem::new();

    info!("hello world");

    let mut cmd_input = CmdInput::new(filesystem);
    let mut prompt_len: usize = print_prompt(&ExitStatus::from_raw(0), &mut stdout).unwrap();
    stdout.flush().unwrap();

    let _ = handle_key(&mut stdout, &mut cmd_input, prompt_len, Key::Char('\t'));
    for c in stdin.keys() {
        if let Ok(val) = c {
            match handle_key(&mut stdout, &mut cmd_input, prompt_len, val) {
                Continue => {}
                CommandStatus(new_status) => {
                    cmd_input.clear();
                    prompt_len = print_prompt(&new_status, &mut stdout).unwrap();
                }
                Exit => break,
            }
        }

        stdout.flush().unwrap();
    }
}
