use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use mercurial::MercurialError;
use perforce::PerforceError;

#[derive(Debug)]
pub enum WorkerError {
    MercurialError { message: String },
    PerforceError { message: String },
}

impl WorkerError {
    pub fn mercurial_error(error: MercurialError) -> WorkerError {
        WorkerError::MercurialError {
            message: error.description().into(),
        }
    }

    pub fn perforce_error(error: PerforceError) -> WorkerError {
        WorkerError::PerforceError {
            message: error.description().into(),
        }
    }
}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            WorkerError::MercurialError { message } => write!(f, "Mercurial error: {}", message),
            WorkerError::PerforceError { message } => write!(f, "Perforce error: {}", message),
        }
    }
}

impl Error for WorkerError {
    fn description(&self) -> &str {
        match self {
            WorkerError::MercurialError { .. } => "Mercurial error",
            WorkerError::PerforceError { .. } => "Perforce error",
        }
    }
}

pub type WorkerResult<T> = Result<T, WorkerError>;
