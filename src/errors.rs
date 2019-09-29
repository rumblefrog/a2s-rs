use std::fmt::{Display, Formatter};

use std::error::Error as StdError;
use std::io::Error as IoError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),

    InvalidResponse,

    MismatchID,

    InvalidBz2Size,

    CheckSumMismatch,

    Other(&'static str),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref inner) => inner.fmt(f),
            _ => f.write_str("dnno"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref inner) => inner.description(),
            Error::InvalidResponse => "Invalid response",
            Error::MismatchID => "Mismatch packet ID",
            Error::InvalidBz2Size => "Invalid Bz2 Size",
            Error::CheckSumMismatch => "Decompressed checksum does not match",
            Error::Other(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}