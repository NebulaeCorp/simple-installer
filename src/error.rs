use std::io;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Game not found")]
    GameNotFound,
    #[error("Directory doesn't exist")]
    MissingDirectory,

    #[error(transparent)]
    Zip(#[from] ZipError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Ini(#[from] ini::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
