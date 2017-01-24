#[macro_use]
extern crate clap;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;
extern crate grep;
extern crate ignore;
extern crate toml;

#[macro_use]
mod macros;
mod errors;
mod args;
mod atty;
mod lints;

use std::error::Error as StdError;

use errors::Error;

fn run() -> Result<(), Error> {
    let matches = match args::parse_args()? {
        Some(m) => m,
        None => return Ok(()),
    };
    let app = args::App::from_args(&matches)?;
    let factory = lints::TomlLintFactory::new_from_path(&app.lint_path)?;
    let lints = factory.build_lints()?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(Error::Argument(ref e)) => {
            wlnerr!("{}", e);
            std::process::exit(2)
        }
        Err(Error::Config(ref e)) => {
            wlnerr!("{}", e);
            std::process::exit(3)
        }
    }
}
