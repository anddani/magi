use git2::Error as Git2Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MagiError {
    IoError(io::Error),
    GitError(Git2Error),
}

impl From<io::Error> for MagiError {
    fn from(error: io::Error) -> Self {
        MagiError::IoError(error)
    }
}

impl From<Git2Error> for MagiError {
    fn from(error: Git2Error) -> Self {
        MagiError::GitError(error)
    }
}

impl fmt::Display for MagiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagiError::IoError(e) => write!(f, "I/O error: {}", e),
            MagiError::GitError(e) => write!(f, "Git error: {}", e),
        }
    }
}

pub type MagiResult<T> = Result<T, MagiError>;
