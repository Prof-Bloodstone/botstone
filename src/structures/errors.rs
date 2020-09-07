use std::error::Error as StdError;
use std::num::ParseIntError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    NumError(ParseIntError),
}
impl StdError for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(msg) => f.write_str(msg),
            Error::NumError(inner) => fmt::Display::fmt(inner, f),
        }
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::NumError(e)
    }
}
