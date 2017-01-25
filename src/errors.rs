extern crate clap;

use std::fmt;
use std::error;
use std::path;
use std::io;

use ignore;
use toml;
use grep;

#[derive(Debug)]
pub enum ArgumentError {
    Clap(clap::Error),
}

impl error::Error for ArgumentError {
    fn description(&self) -> &str {
        match *self {
            ArgumentError::Clap(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ArgumentError::Clap(ref err) => Some(err),
        }
    }
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArgumentError::Clap(ref err) => err.fmt(f),
        }
    }
}

impl From<clap::Error> for ArgumentError {
    fn from(err: clap::Error) -> ArgumentError {
        ArgumentError::Clap(err)
    }
}

#[derive(Debug)]
pub enum SpecificFieldError {
    FieldType { expected: String, actual: String },
    MissingField,
    Ignore(ignore::Error),
    Grep(grep::Error),
}

impl error::Error for SpecificFieldError {
    fn description(&self) -> &str {
        "Invalid field"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SpecificFieldError::FieldType { .. } => None,
            SpecificFieldError::MissingField => None,
            SpecificFieldError::Ignore(ref err) => Some(err),
            SpecificFieldError::Grep(ref err) => Some(err),
        }
    }
}

impl fmt::Display for SpecificFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecificFieldError::FieldType { expected: ref expected, actual: ref actual } => {
                write!(f,
                       "Incorrect type: expected '{}', actual '{}'",
                       expected,
                       actual)
            }
            SpecificFieldError::MissingField => write!(f, "Missing field"),
            SpecificFieldError::Ignore(ref err) => err.fmt(f),
            SpecificFieldError::Grep(ref err) => err.fmt(f),
        }
    }
}

impl From<ignore::Error> for SpecificFieldError {
    fn from(err: ignore::Error) -> SpecificFieldError {
        SpecificFieldError::Ignore(err)
    }
}

impl From<grep::Error> for SpecificFieldError {
    fn from(err: grep::Error) -> SpecificFieldError {
        SpecificFieldError::Grep(err)
    }
}

#[derive(Debug)]
pub struct FieldError {
    field: String,
    error: SpecificFieldError,
}

impl FieldError {
    pub fn new(field: &str, error: SpecificFieldError) -> FieldError {
        FieldError {
            field: field.to_string(),
            error: error,
        }
    }

    pub fn prefix(self, pre: &str) -> FieldError {
        return FieldError {
            field: format!("{}.{}", pre, self.field),
            error: self.error,
        };
    }
}

impl error::Error for FieldError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(&self.error)
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

#[derive(Debug)]
pub enum SpecificConfigError {
    Field(FieldError),
    Io(io::Error),
    Toml(toml::ParserError),
    Ignore(ignore::Error),
}

impl error::Error for SpecificConfigError {
    fn description(&self) -> &str {
        "Invalid config"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SpecificConfigError::Field(ref err) => Some(err),
            SpecificConfigError::Io(ref err) => Some(err),
            SpecificConfigError::Toml(ref err) => Some(err),
            SpecificConfigError::Ignore(ref err) => Some(err),
        }
    }
}

impl fmt::Display for SpecificConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecificConfigError::Io(ref err) => err.fmt(f),
            SpecificConfigError::Field(ref err) => err.fmt(f),
            SpecificConfigError::Toml(ref err) => err.fmt(f),
            SpecificConfigError::Ignore(ref err) => err.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct ConfigError {
    file: Option<String>,
    error: SpecificConfigError,
}

impl ConfigError {
    pub fn add_path(self, file: Option<&path::Path>) -> ConfigError {
        ConfigError {
            file: file.map(|p| p.to_string_lossy().to_string()),
            error: self.error,
        }
    }
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(&self.error)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.file {
            Some(ref file) => write!(f, "{}: {}", file, self.error),
            None => self.error.fmt(f),
        }
    }
}

impl From<FieldError> for ConfigError {
    fn from(err: FieldError) -> ConfigError {
        ConfigError {
            file: None,
            error: SpecificConfigError::Field(err),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError {
            file: None,
            error: SpecificConfigError::Io(err),
        }
    }
}

impl From<toml::ParserError> for ConfigError {
    fn from(err: toml::ParserError) -> ConfigError {
        ConfigError {
            file: None,
            error: SpecificConfigError::Toml(err),
        }
    }
}

impl From<ignore::Error> for ConfigError {
    fn from(err: ignore::Error) -> ConfigError {
        ConfigError {
            file: None,
            error: SpecificConfigError::Ignore(err),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Argument(ArgumentError),
    Config(ConfigError),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Argument(ref err) => err.description(),
            Error::Config(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Argument(ref err) => Some(err),
            Error::Config(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Argument(ref err) => err.fmt(f),
            Error::Config(ref err) => err.fmt(f),
        }
    }
}

impl From<ArgumentError> for Error {
    fn from(err: ArgumentError) -> Error {
        Error::Argument(err)
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Error {
        Error::Config(err)
    }
}
