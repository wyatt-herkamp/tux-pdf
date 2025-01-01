use std::{
    borrow::{Borrow, Cow},
    fmt::Debug,
    ops::Deref,
};

use crate::LowTuxPdfError;

use super::PdfObjectType;

#[inline(always)]
fn encode_name_byte<W>(byte: u8, writer: &mut W) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
{
    if b" \t\n\r\x0C()<>[]{}/%#".contains(&byte) || !(33..=126).contains(&byte) {
        write!(writer, "#{:02X}", byte)?;
    } else {
        writer.write_all(&[byte])?;
    }
    Ok(())
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NameRef([u8]);
impl NameRef {
    pub fn new<B: AsRef<[u8]> + ?Sized>(b: &B) -> &Self {
        unsafe { &*(b.as_ref() as *const [u8] as *const NameRef) }
    }
}
impl AsRef<NameRef> for str {
    fn as_ref(&self) -> &NameRef {
        NameRef::new(self.as_bytes())
    }
}
impl PartialEq<str> for NameRef {
    fn eq(&self, other: &str) -> bool {
        (&self.0) == other.as_bytes()
    }
}
impl PartialEq<Name> for &NameRef {
    fn eq(&self, other: &Name) -> bool {
        self.0 == other.0
    }
}
impl ToOwned for NameRef {
    type Owned = Name;
    fn to_owned(&self) -> Name {
        Name(self.0.to_vec())
    }
}

#[derive(Clone, PartialEq, Eq, Default, Hash)]
pub struct Name(pub Vec<u8>);
impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = std::str::from_utf8(&self.0).unwrap_or("Invalid UTF-8");
        write!(f, "/{} ({:?})", string, self.0)
    }
}
impl Name {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}
impl Deref for Name {
    type Target = NameRef;
    fn deref(&self) -> &Self::Target {
        NameRef::new(&self.0)
    }
}
impl Borrow<NameRef> for Name {
    fn borrow(&self) -> &NameRef {
        NameRef::new(&self.0)
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}
impl From<&[u8]> for Name {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}
impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}
impl From<Vec<u8>> for Name {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}
impl PdfObjectType for Name {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        writer.write_all(b"/")?;
        for byte in self.0 {
            encode_name_byte(byte, writer)?;
        }
        Ok(())
    }
    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writer.write_all(b"/")?;
        for byte in &self.0 {
            encode_name_byte(*byte, writer)?;
        }
        Ok(())
    }
    fn requires_end_separator(&self) -> bool {
        true
    }
    fn requires_separator(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfString {
    /// Using PdfString::Literal directly will not escape the string
    ///
    /// This can be dangerous if the string contains characters that are not allowed in a literal string
    ///
    /// Use [PdfString::literal] or [PdfString::literal_owned] to escape the string
    ///
    /// Using From will have the same effect as using [PdfString::literal]
    Literal(Vec<u8>),
    /// No escaping is required for hexadecimal strings
    ///
    /// The string will be encoded as a hexadecimal string
    Hexadecimal(Vec<u8>),
}
impl From<PdfString> for Vec<u8> {
    fn from(value: PdfString) -> Self {
        match value {
            PdfString::Literal(text) => text,
            PdfString::Hexadecimal(text) => text,
        }
    }
}
impl Default for PdfString {
    fn default() -> Self {
        Self::Literal(Vec::new())
    }
}
impl From<&str> for PdfString {
    fn from(value: &str) -> Self {
        Self::literal(value)
    }
}
impl From<String> for PdfString {
    fn from(value: String) -> Self {
        Self::literal_owned(value.into_bytes())
    }
}

impl PdfString {
    pub fn literal<B: AsRef<[u8]> + ?Sized>(b: &B) -> Self {
        Self::Literal(PdfString::escape_literal(Cow::Borrowed(b.as_ref())))
    }
    pub fn literal_owned<B: Into<Vec<u8>>>(b: B) -> Self {
        Self::Literal(PdfString::escape_literal(Cow::Owned(b.into())))
    }
    pub fn as_slice(&self) -> &[u8] {
        match self {
            PdfString::Literal(text) => text,
            PdfString::Hexadecimal(text) => text,
        }
    }

    fn escape_literal(text: Cow<'_, [u8]>) -> Vec<u8> {
        let mut escape_indice = Vec::new();
        let mut parentheses = Vec::new();
        for (index, &byte) in text.iter().enumerate() {
            match byte {
                b'(' => parentheses.push(index),
                b')' => {
                    if parentheses.pop().is_none() {
                        escape_indice.push(index);
                    }
                }
                b'\\' | b'\r' => escape_indice.push(index),
                _ => continue,
            }
        }
        if escape_indice.is_empty() && parentheses.is_empty() {
            text.into_owned()
        } else {
            escape_indice.extend(parentheses);
            escape_indice.sort();
            let mut escaped = Vec::with_capacity(text.len() + escape_indice.len());
            let mut last_index = 0;
            for index in escape_indice {
                escaped.extend_from_slice(&text[last_index..index]);
                escaped.push(b'\\');
                escaped.push(text[index]);
                last_index = index + 1;
            }
            escaped.extend_from_slice(&text[last_index..]);
            escaped
        }
    }
}

impl PdfObjectType for PdfString {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        self.encode_borrowed(writer)
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
        match self {
            PdfString::Literal(text) => {
                writer.write_all(b"(")?;
                writer.write_all(text)?;
                writer.write_all(b")")?;
            }
            PdfString::Hexadecimal(text) => {
                writer.write_all(b"<")?;
                for byte in text {
                    write!(writer, "{:02X}", byte)?;
                }
                writer.write_all(b">")?;
            }
        }
        Ok(())
    }
    fn type_name(&self) -> &'static str {
        match self {
            PdfString::Literal(_) => "String(Literal)",
            PdfString::Hexadecimal(_) => "String(Hexadecimal)",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_name() {
        use crate::types::PdfObjectType;
        let name = crate::types::string::Name::from("Test");
        let mut buffer = Vec::new();
        name.encode(&mut buffer).unwrap();
        assert_eq!(buffer, b"/Test");
    }
}
