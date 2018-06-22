use time::ParseError as TimeParseError;

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum MercurialError {
    IoError { message: String },
    ExecutionError { message: String },
    CommunicationError { message: String },
    ExitError { exit_code: Option<i32> },
    ChangeParseError { message: String },
    DateFormatError { message: String },
    ReadMetadataError { message: String },
}

impl MercurialError {
    pub fn io_error(error: IoError) -> MercurialError {
        MercurialError::IoError {
            message: error.description().into(),
        }
    }

    pub fn execution_error(error: IoError) -> MercurialError {
        MercurialError::ExecutionError {
            message: error.description().into(),
        }
    }

    pub fn communication_error(error: IoError) -> MercurialError {
        MercurialError::CommunicationError {
            message: error.description().into(),
        }
    }

    pub fn exit_error(exit_code: Option<i32>) -> MercurialError {
        MercurialError::ExitError { exit_code }
    }

    pub fn change_parse_error(error: ParseIntError) -> MercurialError {
        MercurialError::ChangeParseError {
            message: error.description().into(),
        }
    }

    pub fn date_format_error(error: TimeParseError) -> MercurialError {
        MercurialError::DateFormatError {
            message: error.description().into(),
        }
    }

    pub fn read_metadata_error(error: IoError) -> MercurialError {
        MercurialError::ReadMetadataError {
            message: error.description().into(),
        }
    }
}

impl Display for MercurialError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            MercurialError::IoError { message } => write!(f, "IO error: {}", message),
            MercurialError::ExecutionError { message } => write!(f, "Execution error: {}", message),
            MercurialError::CommunicationError { message } => {
                write!(f, "Communication error: {}", message)
            }
            MercurialError::ExitError {
                exit_code: Some(code),
            } => write!(f, "Exit error (code = {})", code),
            MercurialError::ExitError { exit_code: None } => write!(f, "Exit error"),
            MercurialError::ChangeParseError { message } => {
                write!(f, "Change parse error: {}", message)
            }
            MercurialError::DateFormatError { message } => {
                write!(f, "Date format error: {}", message)
            }
            MercurialError::ReadMetadataError { message } => {
                write!(f, "Read metadata error: {}", message)
            }
        }
    }
}

impl Error for MercurialError {
    fn description(&self) -> &str {
        match self {
            MercurialError::IoError { .. } => "IO error",
            MercurialError::ExecutionError { .. } => "Execution error",
            MercurialError::CommunicationError { .. } => "Communication error",
            MercurialError::ExitError { .. } => "Exit error",
            MercurialError::ChangeParseError { .. } => "Change parse error",
            MercurialError::DateFormatError { .. } => "Date format error",
            MercurialError::ReadMetadataError { .. } => "Read metadata error",
        }
    }
}

pub type MercurialResult<T> = Result<T, MercurialError>;
