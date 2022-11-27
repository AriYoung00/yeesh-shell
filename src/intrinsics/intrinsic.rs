use std::borrow::Borrow;
use std::env;
use std::fmt::{Debug, Formatter};

use filesystem::{FileSystem, OsFileSystem};
use lazy_static::lazy_static;

pub trait Intrinsic: Sync + Send {
    fn get_command(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn handler(&self, args: &[String]) -> Result<String, String>;
}

impl Debug for &dyn Intrinsic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("command", &self.get_command())
            .field("description", &self.get_description())
            .finish()
    }
}

impl PartialEq for &dyn Intrinsic {
    fn eq(&self, other: &Self) -> bool {
        (self.get_command() == other.get_command()) && (self.get_description() == other.get_description())
    }
}

struct ChangeDirectory<T: FileSystem + Sync + Send> {
    fs: T,
}

impl<T: FileSystem + Sync + Send> ChangeDirectory<T> {
    pub fn new(filesystem: T) -> Self {
        ChangeDirectory { fs: filesystem }
    }
}

impl<T: FileSystem + Sync + Send> Intrinsic for ChangeDirectory<T> {
    fn get_command(&self) -> &'static str {
        "cd"
    }

    fn get_description(&self) -> &'static str {
        "change the current working directory"
    }

    fn handler(&self, args: &[String]) -> Result<String, String> {
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
    }
}

struct ExitShell(());
impl Intrinsic for ExitShell {
    fn get_command(&self) -> &'static str {
        "exit"
    }

    fn get_description(&self) -> &'static str {
        "exit the shell"
    }

    fn handler(&self, _: &[String]) -> Result<String, String> {
        Ok("So long and thanks for all the fish.".to_string())
    }
}

lazy_static! {
    pub static ref INTRINSICS: Vec<Box<dyn Intrinsic>> = {
        vec![
            Box::new(ChangeDirectory::new(OsFileSystem::new())),
            Box::new(ExitShell(())),
        ]
    };
}

pub fn find_intrinsic(cmd: &String) -> Option<&dyn Intrinsic> {
    INTRINSICS
        .iter()
        .find(|&intrinsic| intrinsic.get_command() == cmd)
        .map(|x| x.borrow())
}
