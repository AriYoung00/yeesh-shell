use std::env;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

pub struct Intrinsic {
    pub command:     &'static str,
    pub description: &'static str,
    pub handler:     fn(args: &[String]) -> Result<String, String>,
}

impl Debug for Intrinsic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Intrinsic")
            .field("command", &self.command)
            .field("description", &self.description)
            .field("handler", &"...")
            .finish()
    }
}

impl PartialEq<Self> for Intrinsic {
    fn eq(&self, other: &Self) -> bool {
        // HACK this is terrible, but it'll work for now
        (self.command == other.command) && (self.description == other.description)
    }
}

pub const INTRINSICS: [Intrinsic; 2] = [
    Intrinsic {
        command:     "cd",
        description: "change the current working directory",
        handler:     |args: &[String]| -> Result<String, String> {
            let path = match args.len() {
                0 => "~",
                1 => args[0].as_str(),
                _ => {
                    return Err("cd: too many arguments".to_string());
                }
            }
            .replace("~", env::var("HOME").or(Err("HOME not set"))?.as_str());

            if let Ok(()) = env::set_current_dir(&path) {
                Ok("".to_string())
            }
            else {
                Err(format!("cd: The directory \"{}\" does not exist\r\n", path))
            }
        },
    },
    Intrinsic {
        command:     "exit",
        description: "exit the shell",
        handler:     |_args: &[String]| Ok("So long and thanks for all the fish.".to_string()),
    },
];

pub fn find_intrinsic(cmd: &String) -> Option<&Intrinsic> {
    for intrinsic in INTRINSICS.iter() {
        if intrinsic.command == cmd {
            return Some(intrinsic);
        }
    }
    None
}