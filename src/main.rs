#[macro_use]
extern crate clap;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;
extern crate grep;
extern crate ignore;
extern crate toml;
extern crate libc;
#[macro_use(slog_error, slog_log)]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate slog_scope;

mod errors;
mod args;
mod ripgrep_stolen;
mod lints;
mod printer;

use std::io;
use errors::Error;
use slog::DrainExt;

fn get_or_log_dir_entry(entry: Result<ignore::DirEntry, ignore::Error>)
                        -> Option<ignore::DirEntry> {
    match entry {
        Ok(dent) => {
            if let Some(err) = dent.error() {
                error!("{}", err);
            }
            let ft = match dent.file_type() {
                None => return Some(dent), // entry is stdin
                Some(ft) => ft,
            };
            // A depth of 0 means the user gave the path explicitly, so we
            // should always try to search it.
            if dent.depth() == 0 && !ft.is_dir() {
                Some(dent)
            } else if ft.is_file() {
                Some(dent)
            } else {
                None
            }
        }
        Err(err) => {
            error!("{}", err);
            None
        }
    }
}

fn run_types<W: io::Write>(printer: &mut printer::IoPrinter<W>,
                           type_defs: &[ignore::types::FileTypeDef])
                           -> Result<(), Error> {
    for def in type_defs {
        printer.type_def(def);
    }
    Ok(())
}

fn run_file_one_thread<W: io::Write>(printer: &mut printer::IoPrinter<W>,
                                     walker: ignore::Walk,
                                     lints: &[lints::Lint])
                                     -> Result<(), Error> {
    for result in walker {
        let dent = match get_or_log_dir_entry(result) {
            Some(dent) => dent,
            None => continue,
        };
        let matched_file = lints.iter()
            .any(|lint| {
                lint.types.is_empty() || lint.types.matched(dent.path(), false).is_whitelist()
            });
        if !matched_file {
            continue;
        }
        printer.path(dent.path());
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
    let mut printer =
        printer::IoPrinter::new(stdout.lock()).use_null(app.printer.null).quiet(app.printer.quiet);

    match app.action {
        args::Action::Search { ref input, ref min_severity, ref output } => {
            let lints = factory.build_lints()?;
            let mut wd = ignore::WalkBuilder::new(&input.paths[0]);
            for path in &input.paths[1..] {
                wd.add(path);
            }
            wd.follow_links(input.follow)
                .hidden(!input.hidden)
                .max_depth(input.maxdepth)
                .git_global(!input.no_ignore && !input.no_ignore_vcs)
                .git_ignore(!input.no_ignore && !input.no_ignore_vcs)
                .git_exclude(!input.no_ignore && !input.no_ignore_vcs)
                .ignore(!input.no_ignore)
                .parents(!input.no_ignore_parent)
                .threads(input.threads);
            match *output {
                args::SearchOutput::None => {
                    if input.threads == 1 || input.is_one_path() {
                        run_file_one_thread(&mut printer, wd.build(), &lints)?;
                    } else {
                        run_file_one_thread(&mut printer, wd.build(), &lints)?;
                    }
                }
                args::SearchOutput::Message => {}
                args::SearchOutput::File { matched } => {}
            }
            Ok(())
        }
        args::Action::PrintTypes => {
            let types = factory.build_types()?;
            run_types(&mut printer, types.definitions())
        }
    }
}

fn main() {
    let drain = slog_term::streamer().compact().build().fuse();
    let root_logger = slog::Logger::root(drain, None);
    slog_scope::set_global_logger(root_logger);
    match run() {
        Ok(_) => {}
        Err(Error::Argument(ref e)) => {
            error!("{}", e);
            std::process::exit(2)
        }
        Err(Error::Config(ref e)) => {
            error!("{}", e);
            std::process::exit(3)
        }
    }
}
