extern crate clap;

use std::fmt;
use std::error;
use std::path;
use std::io;

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
pub enum ConfigError {
    Io { err: io::Error, path: path::PathBuf },
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::Io { ref err, .. } => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::Io { ref err, .. } => Some(err),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io { ref err, .. } => err.fmt(f),
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
