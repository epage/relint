extern crate clap;

use std::vec;
use std::slice;
use std::path;
use std::fs;
use std::io::Read;

use ignore;
use grep;
use toml;

use errors;

fn force_get<'a>(t: &'a toml::Table,
                 field: &str,
                 context: &str)
                 -> Result<&'a toml::Value, errors::ConfigError> {
    t.get(field).ok_or_else(|| {
        errors::ConfigError::Processing { desc: format!("~{} is missing \"{}\"", context, field) }
    })
}

fn force_as_str<'a>(v: &'a toml::Value, context: &str) -> Result<&'a str, errors::ConfigError> {
    v.as_str().ok_or(errors::ConfigError::Processing {
        desc: format!("{} ({}) needs to be a string", context, v),
    })
}

struct FileTypeDef<'a> {
    name: &'a str,
    glob: &'a str,
}

impl<'a> FileTypeDef<'a> {
    fn new_from_value(typedef: &toml::Value) -> Result<FileTypeDef, errors::ConfigError> {
        match *typedef {
            toml::Value::Array(ref a) => {
                if a.len() != 2 {
                    return Err(errors::ConfigError::Processing {
                        desc: format!("File type def ({}) requires [name, glob]", typedef),
                    });
                }
                let name = force_as_str(&a[0], "File type def name")?;
                let glob = force_as_str(&a[1], "File type def glob")?;
                return Ok(FileTypeDef {
                    name: name,
                    glob: glob,
                });
            }
            toml::Value::Table(ref t) => {
                let name = force_as_str(force_get(&t, "name", "File type def")?,
                                        "File type def name")?;
                let glob = force_as_str(force_get(&t, "glob", "File type def")?,
                                        "File type def glob")?;
                return Ok(FileTypeDef {
                    name: name,
                    glob: glob,
                });
            }
            _ => {
                return Err(errors::ConfigError::Processing {
                    desc: format!("Invalid file type def: \"{}\"", typedef),
                });
            }
        }
    }
}

arg_enum! {
    #[derive(Debug)]
    pub enum ErrorLevel {
        Error,
        Warning,
        Info
    }
}

pub struct Lint {
    types: ignore::types::Types,
    severity: ErrorLevel,
    pattern: grep::Grep,
    message: Vec<u8>,
}

impl Lint {
    fn new_from_table(lint: &toml::Table,
                      mut btypes: ignore::types::TypesBuilder,
                      context: &str)
                      -> Result<Lint, errors::ConfigError> {
        if let Some(types) = lint.get("type") {
            match *types {
                toml::Value::String(ref s) => {
                    btypes.select(&s);
                }
                toml::Value::Array(ref a) => {
                    for s in a.iter().map(|v| force_as_str(v, "type")) {
                        let s = s?;
                        btypes.select(&s);
                    }
                }
                _ => {
                    return Err(errors::ConfigError::Processing {
                        desc: format!("{}: Invalid type: \"{}\"", context, types),
                    });
                }
            }
        }

        if let Some(types) = lint.get("type-not") {
            match *types {
                toml::Value::String(ref s) => {
                    btypes.negate(&s);
                }
                toml::Value::Array(ref a) => {
                    for s in a.iter().map(|v| force_as_str(v, "type-not")) {
                        let s = s?;
                        btypes.negate(&s);
                    }
                }
                _ => {
                    return Err(errors::ConfigError::Processing {
                        desc: format!("{}: Invalid type-not: \"{}\"", context, types),
                    });
                }
            }
        }

        let severity =
            lint.get("severity").map(|s| force_as_str(s, "severity")).unwrap_or(Ok("error"))?;
        let severity = severity.parse::<ErrorLevel>()
            .map_err(|s| {
                errors::ConfigError::Processing {
                    desc: format!("{}: Invalid severity: \"{}\"", context, s),
                }
            })?;

        let message = lint.get("message")
            .ok_or(errors::ConfigError::Processing {
                desc: format!("{}: Missing \"message\"", context),
            })?;
        let message = force_as_str(message, "message")?.as_bytes().to_vec();

        let pattern = lint.get("pattern")
            .ok_or(errors::ConfigError::Processing {
                desc: format!("{}: Missing \"pattern\"", context),
            })?;
        let pattern = force_as_str(pattern, "pattern")?;
        let bpattern = grep::GrepBuilder::new(pattern);
        let pattern = bpattern.build()?;

        Ok(Lint {
            types: btypes.build()?,
            severity: severity,
            message: message,
            pattern: pattern,
        })
    }
}

pub struct TomlLintFactory {
    root: toml::Value,
}

impl TomlLintFactory {
    // TODO: compose errors to add context as the stack unwinds
    pub fn new(content: &str) -> Result<TomlLintFactory, errors::ConfigError> {
        let mut parser = toml::Parser::new(content);
        let root = parser.parse()
            .ok_or_else(|| errors::ConfigError::Toml(parser.errors.swap_remove(0)))?;

        Ok(TomlLintFactory { root: toml::Value::Table(root) })
    }

    pub fn new_from_path(lint_path: &path::Path) -> Result<TomlLintFactory, errors::ConfigError> {
        let mut f = fs::File::open(lint_path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        TomlLintFactory::new(&content)
    }

    pub fn build_types(&self) -> Result<ignore::types::TypesBuilder, errors::ConfigError> {
        let mut btypes = ignore::types::TypesBuilder::new();
        btypes.add_defaults();
        let nullArray = toml::Value::Array(Vec::new());
        for def in self.root
            .lookup("relint.types.add")
            .unwrap_or(&nullArray)
            .as_slice()
            .ok_or_else(|| {
                errors::ConfigError::Processing { desc: "Invalid field \"add\"".to_string() }
            })?
            .iter()
            .map(FileTypeDef::new_from_value) {
            let def = def?;
            let name = def.name;
            let glob = def.glob;
            btypes.add(name, glob)?;
        }
        for type_clear in self.root
            .lookup("relint.types.clear")
            .unwrap_or(&nullArray)
            .as_slice()
            .ok_or_else(|| {
                errors::ConfigError::Processing { desc: "Invalid field \"clear\"".to_string() }
            })?
            .iter()
            .map(|s| force_as_str(s, "Type-to-clear")) {
            btypes.clear(type_clear?);
        }
        Ok(btypes)
    }

    pub fn build_lints(&self) -> Result<Vec<Lint>, errors::ConfigError> {
        let lints: Result<Vec<Lint>, errors::ConfigError> = self.root
            .as_table()
            .expect("Table magically became not-a-table?")
            .iter()
            .filter(|kv| kv.0 != "relint")
            .map(|kv| {
                // TODO make ignore::types::TypesBuilder cloneable
                let btypes = self.build_types()?;
                let table = kv.1
                    .as_table()
                    .ok_or_else(|| {
                        errors::ConfigError::Processing {
                            desc: format!("Invalid field \"{}\"", kv.0),
                        }
                    })?;
                Lint::new_from_table(table, btypes, kv.0)
            })
            .collect();
        lints
    }
}
