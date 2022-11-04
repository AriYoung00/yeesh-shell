use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, YeeshError>;

#[derive(Debug)]
pub struct YeeshError {
    details: String
}

impl YeeshError {
    pub fn new(msg: &str) -> YeeshError {
        YeeshError {details: msg.to_string()}
    }
}

impl fmt::Display for YeeshError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for YeeshError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<std::io::Error> for YeeshError {
    fn from(value: std::io::Error) -> Self {
        YeeshError::new(value.to_string().as_str())
    }
}