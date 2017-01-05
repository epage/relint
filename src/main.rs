#[macro_use]
extern crate clap;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

#[macro_use]
mod macros;
mod errors;
mod args;
mod atty;

use std::error::Error as StdError;

use errors::Error;

fn run() -> Result<(), Error> {
    let matches = match args::parse_args()? {
        Some(m) => m,
        None => return Ok(()),
    };
    let app = args::App::from_args(&matches)?;
    Ok(())
}

fn failed(e: Error, code: i32) -> ! {
    wlnerr!("{}", e.description());
    std::process::exit(code)
}

fn main() {
    if let Err(e) = run() {
        match e {
            Error::Argument { .. } => failed(e, 2),
            Error::Config { .. } => failed(e, 3),
        }
    }
}
