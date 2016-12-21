extern crate clap;

#[derive(Debug)]
enum ArgumentError {
    Clap(clap::Error),
}

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ArgumentError::Clap(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ArgumentError {
    fn description(&self) -> &str {
        match *self {
            ArgumentError::Clap(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            ArgumentError::Clap(ref err) => Some(err),
        }
    }
}

impl From<clap::Error> for ArgumentError {
    fn from(err: clap::Error) -> ArgumentError {
        ArgumentError::Clap(err)
    }
}

fn parse_args() -> Result<(), ArgumentError> {
    let arg = |name| {
       clap::Arg::with_name(name)
    };
    let flag = |name| arg(name).long(name);
    let option = |name, value| arg(name).long(name).value_name(value);

    clap::App::new("ReLint")
        .version("0.1")
        .author("Ed Page")
        .about("Custom linting through regular expressions")
        .arg(arg("path").multiple(true))
        .arg(option("config", "FILE")
            .short("c")
            .help("Lints (searches up path if not specified)"))
        .arg(flag("quiet").short("q")
            .help("Do not print any result"))
        .get_matches();
    Ok(())
}

fn run() -> Result<(), ArgumentError> {
    parse_args()
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}
