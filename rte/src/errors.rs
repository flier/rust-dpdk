use std::fmt;
use std::error;

#[derive(Debug, PartialEq, Clone)]
pub enum RteError {
    Init,
}

impl fmt::Display for RteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self).to_string())
    }
}

impl error::Error for RteError {
    fn description(&self) -> &str {
        match self {
            &RteError::Init => "initialize EAL failed.",
        }
    }
}

pub type RteResult<T> = Result<T, RteError>;
