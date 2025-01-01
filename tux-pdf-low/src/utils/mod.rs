use std::io::Write;
pub mod write;
pub struct CountingWriter<W> {
    writer: W,
    count: usize,
}

impl<W> CountingWriter<W> {
    pub fn new(writer: W) -> Self {
        CountingWriter { writer, count: 0 }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<W> Write for CountingWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let count = self.writer.write(buf)?;
        self.count += count;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
