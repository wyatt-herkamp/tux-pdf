use crate::LowTuxPdfError;
mod dictionary;
mod generic_object;
mod object_id;
mod stream;
mod string;
pub use dictionary::*;
pub use generic_object::*;
pub use object_id::*;
pub use stream::*;
pub use string::*;

pub trait PdfType {
    fn write<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write;

    fn write_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write;

    fn size_hint(&self) -> usize {
        0
    }

    fn write_to_vec(self) -> Result<Vec<u8>, LowTuxPdfError>
    where
        Self: Sized,
    {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.size_hint());
        self.write(&mut buffer)?;
        Ok(buffer)
    }
}
impl PdfType for Vec<u8> {
    fn write<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writer.write_all(&self)?;
        Ok(())
    }
    fn size_hint(&self) -> usize {
        self.len()
    }
    fn write_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writer.write_all(self)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfVersion(pub u8, pub u8);
impl From<(u8, u8)> for PdfVersion {
    fn from((major, minor): (u8, u8)) -> Self {
        Self(major, minor)
    }
}
impl Default for PdfVersion {
    fn default() -> Self {
        Self(1, 7)
    }
}
impl PdfType for PdfVersion {
    fn write<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writeln!(writer, "%PDF-{}.{}", self.0, self.1)?;
        Ok(())
    }
    fn write_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        (*self).write(writer)
    }
    fn size_hint(&self) -> usize {
        10
    }
}
macro_rules! copy_encode {
    () => {
        fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
        where
            W: std::io::Write,
        {
            (*self).encode(writer)
        }
    };
}
pub(crate) use copy_encode;
pub trait PdfObjectType {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized;
    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write;
    fn requires_separator(&self) -> bool {
        true
    }

    fn requires_end_separator(&self) -> bool {
        true
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Null;
impl PdfObjectType for Null {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        writer.write_all(b"null")?;
        Ok(())
    }

    copy_encode!();
    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        true
    }
}
impl PdfObjectType for bool {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        writer.write_all(if self { b"true" } else { b"false" })?;
        Ok(())
    }

    copy_encode!();

    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        true
    }
}

impl PdfObjectType for f32 {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(self);
        writer.write_all(s.as_bytes())?;
        Ok(())
    }

    copy_encode!();

    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        true
    }
    fn type_name(&self) -> &'static str {
        "Real"
    }
}
impl PdfObjectType for i64 {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(self);
        writer.write_all(s.as_bytes())?;
        Ok(())
    }
    copy_encode!();

    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        true
    }

    fn type_name(&self) -> &'static str {
        "Number"
    }
}
impl<T> PdfObjectType for Vec<T>
where
    T: PdfObjectType,
{
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        writer.write_all(b"[")?;
        let mut first = true;
        for item in self {
            if first {
                first = false;
            } else if item.requires_separator() {
                writer.write_all(b" ")?;
            }
            item.encode(writer)?;
        }
        writer.write_all(b"]")?;
        Ok(())
    }
    fn requires_end_separator(&self) -> bool {
        false
    }
    fn requires_separator(&self) -> bool {
        false
    }

    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writer.write_all(b"[")?;
        let mut first = true;
        for item in self {
            if first {
                first = false;
            } else if item.requires_separator() {
                writer.write_all(b" ")?;
            }
            item.encode_borrowed(writer)?;
        }
        writer.write_all(b"]")?;
        Ok(())
    }
}
