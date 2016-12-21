#[macro_use]
extern crate clap;

#[macro_use]
mod macros;
mod errors;

use std::io;
use std::io::Write;
use std::error::Error as StdError;
use errors::{Error, ConfigError};

arg_enum! {
    #[derive(Debug)]
    enum StdIn {
        Content,
        Paths
    }
}

fn parse_args() -> Result<(), clap::Error> {
    let arg = |name| {
       clap::Arg::with_name(name)
    };
    let flag = |name| arg(name).long(name);
    let option = |name, value| arg(name).long(name).value_name(value);

    let matches = try!(clap::App::new("ReLint")
        .version("0.1")
        .author("Ed Page")
        .about("Custom linting through regular expressions")
        .arg(arg("path").multiple(true))
        .arg(option("stdin", "FORMAT")
            .possible_values(&StdIn::variants())
            .conflicts_with_all(&["path"])
            .help("Expected format when reading from stdin"))
        .arg(option("config", "FILE")
            .short("c")
            .help("Lints (searches up path if not specified)"))
        .arg(flag("quiet").short("q")
            .help("Do not print any result"))
        .get_matches_safe());
    Ok(())
}

fn run() -> Result<(), Error> {
    let matches = parse_args();
    if let Err(e) = matches {
        if ! e.use_stderr() {
            let out = io::stdout();
            writeln!(&mut out.lock(), "{}", e.description());
            return Ok(())
        }
        return Err(From::from(e));
    }
    Ok(())
}

fn failed(e: Error, code: i32) -> ! {
    wlnerr!("{}", e.description());
    std::process::exit(code)
}

fn main() {
    if let Err(e) = run() {
        match e {
            Error::Argument { .. } => failed(e, 1),
            Error::Config { .. } => failed(e, 2),
        }
    }
}
