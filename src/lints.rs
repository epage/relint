extern crate clap;

use std::path;
use std::fs;
use std::io::Read;

use ignore;
use grep;
use toml;

use errors;

fn force_get<'a>(t: &'a toml::Table, field: &str) -> Result<&'a toml::Value, errors::FieldError> {
    t.get(field)
        .ok_or_else(|| errors::FieldError::new(field, errors::SpecificFieldError::MissingField))
}

fn force_as_str<'a>(v: &'a toml::Value, field: &str) -> Result<&'a str, errors::FieldError> {
    v.as_str().ok_or_else(|| {
        errors::FieldError::new(field,
                                errors::SpecificFieldError::FieldType {
                                    expected: "string".to_string(),
                                    actual: v.type_str().to_string(),
                                })
    })
}

struct FileTypeDef<'a> {
    name: &'a str,
    glob: &'a str,
}

impl<'a> FileTypeDef<'a> {
    fn new_from_table(typedef: &toml::Table) -> Result<FileTypeDef, errors::FieldError> {
        let name = force_get(&typedef, "name")?;
        let name = force_as_str(name, "name")?;
        let glob = force_get(&typedef, "glob")?;
        let glob = force_as_str(glob, "glob")?;
        return Ok(FileTypeDef {
            name: name,
            glob: glob,
        });
    }

    fn new_from_array(typedef: &toml::Array) -> Result<FileTypeDef, errors::FieldError> {
        match typedef.len() {
            0 => {
                Err(errors::FieldError::new("",
                                            errors::SpecificFieldError::FieldType {
                                                expected: "[name, glob]".to_string(),
                                                actual: "[]".to_string(),
                                            }))
            }
            1 => {
                Err(errors::FieldError::new("",
                                            errors::SpecificFieldError::FieldType {
                                                expected: "[name, glob]".to_string(),
                                                actual: "[name]".to_string(),
                                            }))
            }
            2 => {
                let name = force_as_str(&typedef[0], "0")?;
                let glob = force_as_str(&typedef[1], "1")?;
                Ok(FileTypeDef {
                    name: name,
                    glob: glob,
                })
            }
            _ => {
                Err(errors::FieldError::new("",
                                            errors::SpecificFieldError::FieldType {
                                                expected: "[name, glob]".to_string(),
                                                actual: "[name, glob, ...]".to_string(),
                                            }))
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
                      mut btypes: ignore::types::TypesBuilder)
                      -> Result<Lint, errors::FieldError> {
        if let Some(types) = lint.get("type") {
            match *types {
                toml::Value::String(ref s) => {
                    btypes.select(&s);
                }
                toml::Value::Array(ref a) => {
                    for s in a.iter().map(|v| force_as_str(v, "type[...]")) {
                        let s = s?;
                        btypes.select(&s);
                    }
                }
                _ => {
                    return Err(errors::FieldError::new("type-not",
                                                       errors::SpecificFieldError::FieldType {
                                                           expected: "string or string-array"
                                                               .to_string(),
                                                           actual: types.type_str().to_string(),
                                                       }));
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
                    return Err(errors::FieldError::new("type-not",
                                                       errors::SpecificFieldError::FieldType {
                                                           expected: "string or string-array"
                                                               .to_string(),
                                                           actual: types.type_str().to_string(),
                                                       }));
                }
            }
        }

        let severity =
            lint.get("severity").map(|s| force_as_str(s, "severity")).unwrap_or(Ok("error"))?;
        let severity = severity.parse::<ErrorLevel>()
            .map_err(|s| {
                errors::FieldError::new("severity",
                                        errors::SpecificFieldError::FieldType {
                                            expected: s,
                                            actual: severity.to_string(),
                                        })
            })?;

        let message = force_get(lint, "message")?;
        let message = force_as_str(message, "message")?.as_bytes().to_vec();

        let pattern = force_get(lint, "pattern")?;
        let pattern = force_as_str(pattern, "pattern")?;
        let bpattern = grep::GrepBuilder::new(pattern);
        let pattern = bpattern.build()
            .map_err(|e| errors::FieldError::new("severity", errors::SpecificFieldError::Grep(e)))?;

        Ok(Lint {
            types:
                btypes.build()
                .map_err(|e| errors::FieldError::new("...", errors::SpecificFieldError::Ignore(e)))?,
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
    pub fn new(content: &str) -> Result<TomlLintFactory, errors::ConfigError> {
        let mut parser = toml::Parser::new(content);
        let root = parser.parse()
            .ok_or_else(|| parser.errors.swap_remove(0))?;

        Ok(TomlLintFactory { root: toml::Value::Table(root) })
    }

    pub fn new_from_path(lint_path: &path::Path) -> Result<TomlLintFactory, errors::ConfigError> {
        let mut f = fs::File::open(lint_path).map_err(|e| {
                errors::ConfigError::from(e).add_path(lint_path.to_string_lossy().to_string())
            })?;
        let mut content = String::new();
        f.read_to_string(&mut content)
            .map_err(|e| {
                errors::ConfigError::from(e).add_path(lint_path.to_string_lossy().to_string())
            })?;
        TomlLintFactory::new(&content)
    }

    pub fn build_types(&self) -> Result<ignore::types::TypesBuilder, errors::ConfigError> {
        let mut btypes = ignore::types::TypesBuilder::new();
        btypes.add_defaults();
        let null_array = toml::Value::Array(Vec::new());
        let adds = self.root
            .lookup("relint.types.add")
            .unwrap_or(&null_array);
        let adds = adds.as_slice()
            .ok_or_else(|| {
                errors::FieldError::new("relint.types.add",
                                        errors::SpecificFieldError::FieldType {
                                            expected: "array".to_string(),
                                            actual: adds.type_str().to_string(),
                                        })
            })?;
        for def in adds {
            let def = match *def {
                toml::Value::Table(ref t) => FileTypeDef::new_from_table(t),
                toml::Value::Array(ref a) => FileTypeDef::new_from_array(a),
                _ => {
                    Err(errors::FieldError::new("relint.types.add[...]",
                                                errors::SpecificFieldError::FieldType {
                                                    expected: "table/array".to_string(),
                                                    actual: def.type_str().to_string(),
                                                }))
                }
            }?;
            let name = def.name;
            let glob = def.glob;
            btypes.add(name, glob)
                .map_err(|e| {
                    errors::FieldError::new("relint.types.add[...]",
                                            errors::SpecificFieldError::Ignore(e))
                })?;
        }
        let clears = self.root
            .lookup("relint.types.clear")
            .unwrap_or(&null_array);
        let clears = clears.as_slice()
            .ok_or_else(|| {
                errors::FieldError::new("relint.types.clear",
                                        errors::SpecificFieldError::FieldType {
                                            expected: "array".to_string(),
                                            actual: clears.type_str().to_string(),
                                        })
            })?;
        for type_clear in clears.iter()
            .map(|s| force_as_str(s, "relint.types.clear[...]")) {
            btypes.clear(type_clear?);
        }
        Ok(btypes)
    }

    fn build_lint(&self,
                  check_name: &str,
                  settings: &toml::Value)
                  -> Result<Lint, errors::ConfigError> {
        // TODO make ignore::types::TypesBuilder cloneable
        let btypes = self.build_types()?;
        let settings = settings.as_table()
            .ok_or_else(|| {
                errors::FieldError::new(check_name,
                                        errors::SpecificFieldError::FieldType {
                                            expected: "table".to_string(),
                                            actual: settings.type_str().to_string(),
                                        })
            })?;
        let lint = Lint::new_from_table(settings, btypes)?;
        Ok(lint)
    }

    pub fn build_lints(&self) -> Result<Vec<Lint>, errors::ConfigError> {
        let lints: Result<Vec<Lint>, errors::ConfigError> = self.root
            .as_table()
            .expect("Table magically became not-a-table?")
            .iter()
            .filter(|kv| kv.0 != "relint")
            .map(|kv| self.build_lint(kv.0, kv.1))
            .collect();
        lints
    }
}
