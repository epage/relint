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
mod ripgrep_stolen;
mod lints;
mod printer;

use std::io;

use errors::Error;

fn run_types<W: io::Write>(printer: &mut Option<&mut printer::IoPrinter<W>>,
                           type_defs: &[ignore::types::FileTypeDef])
                           -> Result<(), Error> {
    for def in type_defs {
        match *printer {
            Some(ref mut p) => p.type_def(def),
            None => {}
        }
    }
    Ok(())
}

fn run() -> Result<(), Error> {
    let matches = match args::parse_args()? {
        Some(m) => m,
        None => return Ok(()),
    };
    let app = args::App::from_args(&matches)?;
    let factory = lints::TomlLintFactory::new_from_path(&app.lint_path)?;

    let stdout = std::io::stdout();
    let mut printer = match app.printer.quiet {
        true => None,
        false => {
            let mut printer = printer::IoPrinter::new(stdout.lock());
            if app.printer.null {
                printer = printer.null();
            }
            Some(printer)
        }
    };

    match app.action {
        args::Action::Search { ref input, ref min_severity, ref output } => {
            let lints = factory.build_lints()?;
            Ok(())
        }
        args::Action::PrintTypes => {
            let types = factory.build_types()?;
            run_types(&mut printer.as_mut(), types.definitions())
        }
    }
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
