use std::io;
use std::io::Write;
use std::process::ExitStatus;

use termion::cursor::DetectCursorPos;
use termion::{color, cursor};

pub fn print_prompt<T>(status: &ExitStatus, out: &mut T) -> io::Result<usize>
where
    T: Write,
{
    let path = if let Ok(dir) = std::env::current_dir() {
        format!("{}", dir.display())
    }
    else {
        "".to_string()
    };
    write!(out, "{}{}", color::Fg(color::Reset), path)?;
    if status.success() {
        write!(out, "{}", color::Fg(color::Green))?;
    }
    else {
        write!(out, "{}", color::Fg(color::Red))?;
    }
    let prompt = format!("> ",);

    write!(out, "{}", prompt)?;
    write!(out, "{}", color::Fg(color::Reset))?;
    Ok(prompt.len() + path.len())
}
