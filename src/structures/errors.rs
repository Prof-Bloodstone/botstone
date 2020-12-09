use json5::Error as Json5Error;
use std::num::ParseIntError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum BotstoneError {
    #[error("error parsing")]
    ParseError(#[from] ParseError),
    #[error("database error")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(ThisError, Debug)]
pub enum ParseError {
    #[error(transparent)]
    ColourParseError(#[from] ColourParseError),
    #[error("invalid number `{0:?}`, cause by `{1:?}`")]
    InvalidNumber(String, ParseIntError),
    #[error("invalid json `{0:?}`")]
    InvalidJson(Json5Error),
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

#[derive(ThisError, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("nothing was deleted")]
    NothingDeleted,
}
