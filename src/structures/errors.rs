use std::num::ParseIntError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum BotstoneError {
    #[error("error parsing")]
    ParseError(#[from] ParseError),
}

#[derive(ThisError, Debug)]
pub enum ParseError {
    #[error(transparent)]
    ColourParseError(#[from] ColourParseError),
}

#[derive(ThisError, Debug)]
pub enum ColourParseError {
    #[error("invalid colour hex length: `{0:?}`")]
    InvalidColourHexLength(String),
    #[error("invalid colour hex value: `{0:?}`, caused by: `{1:?}`")]
    InvalidColourHexValue(String, ParseIntError),
    #[error("unknown colour name: `{0:?}`")]
    UnknownColourName(String),
}
