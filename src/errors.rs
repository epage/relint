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
pub enum ConfigError {
    Io(io::Error),
    Toml(toml::ParserError),
    Ignore(ignore::Error),
    Grep(grep::Error),
    Processing { desc: String },
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::Io(ref err) => err.description(),
            ConfigError::Toml(ref err) => err.description(),
            ConfigError::Ignore(ref err) => err.description(),
            ConfigError::Grep(ref err) => err.description(),
            ConfigError::Processing { ref desc } => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::Io(ref err) => Some(err),
            ConfigError::Toml(ref err) => Some(err),
            ConfigError::Ignore(ref err) => Some(err),
            ConfigError::Grep(ref err) => Some(err),
            ConfigError::Processing { .. } => None,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) => err.fmt(f),
            ConfigError::Toml(ref err) => err.fmt(f),
            ConfigError::Ignore(ref err) => err.fmt(f),
            ConfigError::Grep(ref err) => err.fmt(f),
            ConfigError::Processing { ref desc } => desc.fmt(f),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<toml::ParserError> for ConfigError {
    fn from(err: toml::ParserError) -> ConfigError {
        ConfigError::Toml(err)
    }
}

impl From<ignore::Error> for ConfigError {
    fn from(err: ignore::Error) -> ConfigError {
        ConfigError::Ignore(err)
    }
}

impl From<grep::Error> for ConfigError {
    fn from(err: grep::Error) -> ConfigError {
        ConfigError::Grep(err)
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
