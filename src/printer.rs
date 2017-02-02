use std::io;
use std::path;

use ignore;

use ripgrep_stolen::pathutil;

pub struct IoPrinter<W> {
    writer: W,
    sep: u8,
}

#[cfg(unix)]
fn path_bytes<'a>(path: &'a path::Path) -> &'a [u8] {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str().as_bytes()
}

static PLACEHOLDER: &'static str = "<INVALID>";

#[cfg(not(unix))]
fn path_bytes(path: &path::Path) -> &[u8] {
    path.to_str().unwrap_or(PLACEHOLDER).as_bytes()
}

impl<W: io::Write> IoPrinter<W> {
    pub fn new(writer: W) -> IoPrinter<W> {
        IoPrinter {
            writer: writer,
            sep: b'\n',
        }
    }

    pub fn null(mut self) -> IoPrinter<W> {
        self.sep = b'\0';
        self
    }

    pub fn type_def(&mut self, type_def: &ignore::types::FileTypeDef) {
        self.write(type_def.name().as_bytes());
        self.write(b": ");
        let mut first = true;
        for glob in type_def.globs() {
            if !first {
                self.write(b", ");
            }
            self.write(glob.as_bytes());
            first = false;
        }
        self.write_sep();
    }

    pub fn path(&mut self, path: &path::Path) {
        self.write(path_bytes(pathutil::strip_prefix("./", path).unwrap_or(path)));
        self.write_sep();
    }

    fn write(&mut self, buf: &[u8]) {
        let _ = self.writer.write_all(buf);
    }

    fn write_sep(&mut self) {
        let sep = self.sep;
        self.write(&[sep]);
    }
}
