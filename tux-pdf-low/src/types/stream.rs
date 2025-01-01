use crate::LowTuxPdfError;

use super::{dictionary::Dictionary, DictionaryIoWriter, DictionaryType, PdfObjectType, PdfType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream<D = Dictionary, Content: PdfType = Vec<u8>> {
    /// The dictionary of the stream
    pub dictionary: D,
    /// The content of the stream
    pub content: Content,
    /// Whether the stream allows compression
    pub allows_compression: bool,
    /// The start position of the stream
    pub start_position: Option<usize>,
}
impl<D, Content: PdfType> Stream<D, Content> {
    pub fn encode_content_to_vec(self) -> Result<Stream<D, Vec<u8>>, LowTuxPdfError> {
        let new_content = self.content.write_to_vec()?;
        Ok(Stream {
            dictionary: self.dictionary,
            content: new_content,
            allows_compression: self.allows_compression,
            start_position: self.start_position,
        })
    }
    pub fn with_compression(self, value: bool) -> Self {
        Stream {
            allows_compression: value,
            ..self
        }
    }
}
impl<D: DictionaryType, Content: PdfType> From<Stream<D, Content>> for Stream<Dictionary, Content> {
    fn from(stream: Stream<D, Content>) -> Self {
        Stream {
            dictionary: stream.dictionary.into_dictionary(),
            content: stream.content,
            allows_compression: stream.allows_compression,
            start_position: stream.start_position,
        }
    }
}
impl<D, Content: PdfType> Default for Stream<D, Content>
where
    D: Default,
    Content: Default,
{
    fn default() -> Self {
        Stream {
            dictionary: D::default(),
            content: Content::default(),
            allows_compression: true,
            start_position: None,
        }
    }
}
impl<Content: PdfType, D> Stream<D, Content> {
    pub fn new(dict: D, content: Content) -> Stream<D, Content> {
        Stream {
            dictionary: dict,
            content,
            allows_compression: true,
            start_position: None,
        }
    }
}
impl<Content: PdfType> PdfObjectType for Stream<Dictionary, Content> {
    fn encode<W>(mut self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        self.dictionary
            .set("Length", self.content.size_hint() as i64);
        self.dictionary.encode(writer)?;
        writer.write_all(b"\n")?;
        writer.write_all(b"stream\n")?;
        self.content.write(writer)?;
        writer.write_all(b"\nendstream")?;
        Ok(())
    }

    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        self.dictionary.encode_borrowed(writer)?;
        writer.write_all(b"\n")?;
        writer.write_all(b"stream\n")?;
        self.content.write_borrowed(writer)?;
        writer.write_all(b"\nendstream")?;
        Ok(())
    }
    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        false
    }
}
impl<Content: PdfType, D> PdfObjectType for Stream<D, Content>
where
    D: DictionaryType,
{
    fn encode<W>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        {
            let dictionary_writer = DictionaryIoWriter { writer };

            self.dictionary.write_to_dictionary(dictionary_writer)?;
        }
        writer.write_all(b"\n")?;
        writer.write_all(b"stream\n")?;
        self.content.write(writer)?;
        writer.write_all(b"\nendstream")?;
        Ok(())
    }

    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        {
            let dictionary_writer = DictionaryIoWriter { writer };

            self.dictionary
                .write_to_dictionary_borrowed(dictionary_writer)?;
        }
        writer.write_all(b"\n")?;
        writer.write_all(b"stream\n")?;
        self.content.write_borrowed(writer)?;
        writer.write_all(b"\nendstream")?;
        Ok(())
    }
    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        false
    }
}
