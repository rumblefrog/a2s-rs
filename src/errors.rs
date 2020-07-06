use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Mismatch packet ID")]
    MismatchID,

    #[error("Invalid Bz2 size")]
    InvalidBz2Size,

    #[error("Decompressed checksum does not match")]
    CheckSumMismatch,

    #[error("{0}")]
    Other(&'static str),
}
