use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

use serde_yaml::Error as YamlError;

#[derive(Debug)]
pub enum ConfigError {
    IoError { message: String },
    DeserializationError { message: String },
}

impl ConfigError {
    pub fn io_error(error: IoError) -> ConfigError {
        ConfigError::IoError {
            message: error.description().into(),
        }
    }

    pub fn deserialization_error(error: YamlError) -> ConfigError {
        ConfigError::DeserializationError {
            message: error.description().into(),
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ConfigError::IoError { message } => write!(f, "IO error: {}", message),
            ConfigError::DeserializationError { message } => {
                write!(f, "Deserialization error: {}", message)
            }
        }
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match self {
            ConfigError::IoError { .. } => "IO error",
            ConfigError::DeserializationError { .. } => "Deserialization error",
        }
    }
}

pub type ConfigResult<T> = Result<T, ConfigError>;
