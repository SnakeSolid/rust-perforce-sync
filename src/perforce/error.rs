use time::ParseError as TimeParseError;

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum PerforceError {
    IoError { message: String },
    ExecutionError { message: String },
    CommunicationError { message: String },
    ExitError { exit_code: Option<i32> },
    IncorrectChange { commit: u32 },
    LoginFailed,
    NotLoggedIn,
    ParsingChangeFailed,
    DateParseError { message: String },
}

impl PerforceError {
    pub fn io_error(error: IoError) -> PerforceError {
        PerforceError::IoError {
            message: error.description().into(),
        }
    }

    pub fn execution_error(error: IoError) -> PerforceError {
        PerforceError::ExecutionError {
            message: error.description().into(),
        }
    }

    pub fn communication_error(error: IoError) -> PerforceError {
        PerforceError::CommunicationError {
            message: error.description().into(),
        }
    }

    pub fn exit_error(exit_code: Option<i32>) -> PerforceError {
        PerforceError::ExitError { exit_code }
    }

    pub fn incorrect_change(commit: u32) -> PerforceError {
        PerforceError::IncorrectChange { commit }
    }

    pub fn date_parse_error(error: TimeParseError) -> PerforceError {
        PerforceError::DateParseError {
            message: error.description().into(),
        }
    }
}

impl From<ParseIntError> for PerforceError {
    fn from(_: ParseIntError) -> PerforceError {
        PerforceError::ParsingChangeFailed
    }
}

impl Display for PerforceError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            PerforceError::IoError { .. } => write!(f, "IO error"),
            PerforceError::ExecutionError { .. } => write!(f, "Execution error"),
            PerforceError::CommunicationError { .. } => write!(f, "Communication error"),
            PerforceError::ExitError { .. } => write!(f, "Exit error"),
            PerforceError::IncorrectChange { .. } => write!(f, "Incorrect change"),
            PerforceError::LoginFailed => write!(f, "Login failed"),
            PerforceError::NotLoggedIn => write!(f, "Not logged in"),
            PerforceError::ParsingChangeFailed => write!(f, "Parsing change failed"),
            PerforceError::DateParseError { .. } => write!(f, "Time parse error"),
        }
    }
}

impl Error for PerforceError {
    fn description(&self) -> &str {
        match self {
            PerforceError::IoError { .. } => "IO error",
            PerforceError::ExecutionError { .. } => "Execution error",
            PerforceError::CommunicationError { .. } => "Communication error",
            PerforceError::ExitError { .. } => "Exit error",
            PerforceError::IncorrectChange { .. } => "Incorrect change",
            PerforceError::LoginFailed => "Login failed",
            PerforceError::NotLoggedIn => "Not logged in",
            PerforceError::ParsingChangeFailed => "Parsing change failed",
            PerforceError::DateParseError { .. } => "Time parse error",
        }
    }
}

pub type PerforceResult<T> = Result<T, PerforceError>;
