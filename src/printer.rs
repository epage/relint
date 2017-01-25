use std::io;

use ignore;

pub struct IoPrinter<W> {
    writer: W,
    sep: u8,
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

    fn write(&mut self, buf: &[u8]) {
        let _ = self.writer.write_all(buf);
    }

    fn write_sep(&mut self) {
        let sep = self.sep;
        self.write(&[sep]);
    }
}
