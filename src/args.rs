extern crate clap;

use std::str;
use std::io;
use std::io::Write;
use std::path;
use std::fmt;
use std::env;
use std::error::Error as StdError;

use lints;
use atty;
use errors;

static CWD: &'static str = "./";
static STDIN: &'static str = "-";
static DEFAULT_CONFIG_FILE: &'static str = "relint.toml";

struct PathWalkSource {
    paths: Vec<path::PathBuf>,
    follow: bool,
    hidden: bool,
    no_ignore: bool,
    no_ignore_vcs: bool,
    maxdepth: Option<usize>,
    threads: usize,
}

impl PathWalkSource {
    fn from_args(paths: Vec<path::PathBuf>,
                 matches: &clap::ArgMatches)
                 -> Result<PathWalkSource, errors::ArgumentError> {
        let source = PathWalkSource {
            paths: paths,
            follow: matches.is_present("follow"),
            hidden: matches.is_present("hidden"),
            no_ignore: matches.is_present("no-ignore"),
            no_ignore_vcs: matches.is_present("no-ignore-vcs"),
            maxdepth: parsed_value_of(matches, "maxdepth")?,
            threads: parsed_value_of(matches, "threads")?.unwrap_or(0),
        };
        Ok(source)
    }
}

enum InputSource {
    PathWalk(PathWalkSource),
    StdIn,
}

impl InputSource {
    fn from_args(matches: &clap::ArgMatches) -> Result<InputSource, errors::ArgumentError> {
        let paths: Vec<path::PathBuf> = match matches.values_of("path") {
            None => vec![default_path()],
            Some(vals) => vals.map(|p| path::Path::new(p).to_path_buf()).collect(),
        };
        if is_stdin_requested(&paths)? {
            return Ok(InputSource::StdIn);
        } else {
            return Ok(InputSource::PathWalk(PathWalkSource::from_args(paths, matches)?));
        }
    }
}

pub struct App {
    input: InputSource,
    pub lint_path: path::PathBuf,
}

impl App {
    pub fn from_args(matches: &clap::ArgMatches) -> Result<App, errors::ArgumentError> {
        let lint_path = get_project_file(&matches, "lints", DEFAULT_CONFIG_FILE)?;

        let input_source = InputSource::from_args(&matches)?;

        let app = App {
            input: input_source,
            lint_path: lint_path,
        };

        Ok(app)
    }
}

fn arg(name: &str) -> clap::Arg {
    clap::Arg::with_name(name)
}

fn flag(name: &str) -> clap::Arg {
    clap::Arg::with_name(name).long(name)
}

fn option<'a>(name: &'a str, value: &'a str) -> clap::Arg<'a, 'a> {
    clap::Arg::with_name(name).long(name).value_name(value)
}

fn validate_number(s: String) -> Result<(), String> {
    s.parse::<usize>().map(|_| ()).map_err(|err| err.to_string())
}

fn parsed_value_of<F: str::FromStr>(matches: &clap::ArgMatches,
                                    name: &str)
                                    -> Result<Option<F>, errors::ArgumentError>
    where <F as str::FromStr>::Err: fmt::Debug
{
    match matches.value_of(name) {
        None => Ok(None),
        Some(v) => {
            Ok(v.parse()
                .map(Some)
                .expect("Input should have already been validated"))
        }
    }
}

fn build_app<'a>() -> clap::App<'a, 'a> {
    clap::App::new("relint")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Custom linting through regular expressions")
        .arg(arg("path")
            .multiple(true)
            .default_value(CWD)
            .help("Specify '-' for stdin"))
        .arg(flag("follow")
            .short("L")
            .help("Follow symbolic links."))
        .arg(flag("hidden").help("Search hidden files and directories."))
        .arg(flag("no-ignore").help("Don't respect ignore files."))
        .arg(flag("no-ignore-vcs").help("Don't respect VCS ignore files"))
        .arg(option("maxdepth", "NUM")
            .validator(validate_number)
            .help("Descend at most NUM directories."))
        .group(clap::ArgGroup::with_name("Paths")
            .args(&["follow", "no-ignore", "no-ignore-vcs", "maxdepth"])
            .multiple(true))
        .arg(option("lints", "FILE")
            .short("c")
            .help("Lints (searches up path if not specified)"))
        .arg(option("error-level", "LEVEL")
            .possible_values(&lints::ErrorLevel::variants())
            .default_value("Error")
            .help("Lint item level to be treated as errors"))
        .arg(flag("files-with-errors")
            .short("l")
            .help("Only show the path of each file with at least one match."))
        .arg(flag("files-without-errors")
            .help("Only show the path of each file that contains zero matches."))
        .arg(flag("files").help("Print each file that would be searched."))
        .group(clap::ArgGroup::with_name("PrintNames")
            .args(&["files", "files-with-errors", "files-without-errors"]))
        .arg(flag("null")
            .requires("PrintNames")
            .help("Print NUL byte after file names"))
        .arg(flag("quiet")
            .short("q")
            .help("Do not print any result"))
        .arg(flag("type-list").help("Show all supported file types."))
        .arg(option("threads", "NUM")
            .short("j")
            .validator(validate_number))
}

pub fn parse_args<'a>() -> Result<Option<clap::ArgMatches<'a>>, errors::ArgumentError> {
    match build_app().get_matches_safe() {
        Ok(m) => Ok(Some(m)),
        Err(e) => {
            if !e.use_stderr() {
                let out = io::stdout();
                writeln!(&mut out.lock(), "{}", e.description()).expect("How does this fail?");
                return Ok(None);
            }
            Err(From::from(e))
        }
    }
}

fn find_project_file(dir: &path::Path, name: &str) -> Option<path::PathBuf> {
    let mut file_path = dir.join(name);
    while !file_path.exists() {
        file_path.pop();
        if file_path.parent() == None {
            return None;
        }
        file_path.set_file_name(name);
    }
    Some(file_path)
}

fn get_project_file(matches: &clap::ArgMatches,
                    name: &str,
                    default: &str)
                    -> Result<path::PathBuf, errors::ArgumentError> {
    let cwd = env::current_dir().expect("How does this fail?");
    let path = matches.value_of(name)
        .map(|p| Some(path::Path::new(p).to_path_buf()))
        .unwrap_or_else(|| find_project_file(cwd.as_path(), DEFAULT_CONFIG_FILE))
        .ok_or_else(|| {
            clap::Error::with_description(&format!("The following required argument was not \
                                                    provided: --{}",
                                                   name),
                                          clap::ErrorKind::MissingRequiredArgument)
        })?;
    Ok(path)
}

fn default_path() -> path::PathBuf {
    let search_cwd = atty::on_stdin() || !atty::stdin_is_readable();
    let default_path = match search_cwd {
        true => CWD,
        false => STDIN,
    };
    path::Path::new(default_path).to_path_buf()
}

fn is_stdin_requested(paths: &[path::PathBuf]) -> Result<bool, errors::ArgumentError> {
    if paths.len() == 1 {
        return Ok(paths[0] == path::Path::new(STDIN));
    } else {
        let stdin_path = path::Path::new(STDIN).to_path_buf();
        if paths.contains(&stdin_path) {
            return Err(clap::Error::with_description("Cannot mix stdin (-) with file paths",
                                                     clap::ErrorKind::ArgumentConflict))?;
        }
        return Ok(false);
    }
}
