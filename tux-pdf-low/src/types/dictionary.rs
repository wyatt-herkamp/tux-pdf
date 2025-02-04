use indexmap::IndexMap;
pub mod stream_header;
pub mod trailer;
use crate::LowTuxPdfError;

use super::{string::Name, NameRef, Object, PdfObjectType};
pub static START_DICTIONARY: &[u8] = b"<<";
pub static END_DICTIONARY: &[u8] = b">>";

#[macro_export]
macro_rules! dictionary {
    () => {
        $crate::types::Dictionary::new()
    };
    (
        $($key:expr => $value:expr),*
    ) => {
        {
            let mut dict = $crate::types::Dictionary::new();
            $(
                dict.set($key, $value);
            )*
            dict
        }
    };
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dictionary(IndexMap<Name, Object>);

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_type(&mut self, value: impl Into<Name>) {
        self.set("Type", value.into());
    }
    pub fn set(&mut self, key: impl Into<Name>, value: impl Into<Object>) {
        self.0.insert(key.into(), value.into());
    }
    pub fn get(&self, key: impl AsRef<NameRef>) -> Option<&Object> {
        self.0.get(key.as_ref())
    }
    pub fn get_or_err(&self, key: &str) -> Result<&Object, LowTuxPdfError> {
        self.get(key)
            .ok_or_else(|| LowTuxPdfError::MissingDictionaryKey(key.to_string()))
    }
    /// Gets the Type parameter of the dictionary
    ///
    /// If the Type parameter is not present, it defaults to "Linearized"
    ///
    /// # Errors
    /// Returns an error if the Type parameter is not a Name
    pub fn dictionary_type(&self) -> Result<Option<Name>, LowTuxPdfError> {
        let Some(value) = self.get("Type") else {
            return Ok(None);
        };
        match value {
            Object::Name(name) => Ok(Some(name.clone())),
            _ => Err(LowTuxPdfError::InvalidDictionaryType {
                expected: "Name",
                actual: value.type_name(),
            }),
        }
    }
    pub fn writer(&mut self) -> DictionaryWriterToDictionary {
        DictionaryWriterToDictionary { dict: self }
    }
}
impl IntoIterator for Dictionary {
    type Item = (Name, Object);
    type IntoIter = indexmap::map::IntoIter<Name, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl FromIterator<(Name, Object)> for Dictionary {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Name, Object)>,
    {
        let mut dict = Dictionary::default();
        for (key, value) in iter {
            dict.0.insert(key, value);
        }
        dict
    }
}

impl PdfObjectType for Dictionary {
    fn encode<W>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        writer.write_all(START_DICTIONARY)?;
        for (key, value) in self.0 {
            key.encode(writer)?;
            if value.requires_separator() {
                writer.write_all(b" ")?;
            }
            value.encode(writer)?;
        }
        writer.write_all(END_DICTIONARY)?;
        Ok(())
    }

    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        writer.write_all(START_DICTIONARY)?;
        for (key, value) in &self.0 {
            key.encode_borrowed(writer)?;
            if value.requires_separator() {
                writer.write_all(b" ")?;
            }
            value.encode_borrowed(writer)?;
        }
        writer.write_all(END_DICTIONARY)?;
        Ok(())
    }
    fn requires_end_separator(&self) -> bool {
        false
    }
    fn requires_separator(&self) -> bool {
        false
    }
}
/// A type that can target of a dictionary write. Must start with start_dictionary and end with end_dictionary
pub trait WritableDictionary {
    fn write_value(
        &mut self,
        key: impl Into<Name>,
        value: impl Into<Object>,
    ) -> Result<(), crate::LowTuxPdfError>;

    fn start_dictionary(&mut self) -> Result<(), crate::LowTuxPdfError> {
        Ok(())
    }

    fn end_dictionary(self) -> Result<(), crate::LowTuxPdfError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
pub trait DictionaryType {
    fn get_type(&self) -> Option<&NameRef>;

    fn from_dictionary(dict: &mut Dictionary) -> Result<Self, crate::LowTuxPdfError>
    where
        Self: Sized;
    /// Write the contents of this object to a dictionary
    ///
    /// Do not forget to call start_dictionary before writing the contents
    ///
    /// Do not forget to call end_dictionary after writing the contents
    fn write_to_dictionary<WD>(self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary;
    fn write_to_dictionary_borrowed<WD>(&self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary;

    fn into_dictionary(self) -> Dictionary
    where
        Self: Sized,
    {
        let mut dict = Dictionary::default();
        self.write_to_dictionary(&mut dict.writer()).unwrap();
        dict
    }
}

pub struct DictionaryWriterToDictionary<'dict> {
    dict: &'dict mut Dictionary,
}
impl WritableDictionary for DictionaryWriterToDictionary<'_> {
    fn write_value(
        &mut self,
        key: impl Into<Name>,
        value: impl Into<Object>,
    ) -> Result<(), crate::LowTuxPdfError> {
        self.dict.set(key, value);
        Ok(())
    }
}

///Allows you to write the contents of a dictionary directly to a writer
#[derive(Debug, PartialEq)]
pub struct DictionaryIoWriter<'writer, W>
where
    W: std::io::Write,
{
    pub writer: &'writer mut W,
}
impl<'writer, W> From<&'writer mut W> for DictionaryIoWriter<'writer, W>
where
    W: std::io::Write,
{
    fn from(writer: &'writer mut W) -> Self {
        Self { writer }
    }
}

impl<W> WritableDictionary for DictionaryIoWriter<'_, W>
where
    W: std::io::Write,
{
    fn write_value(
        &mut self,
        key: impl Into<Name>,
        value: impl Into<Object>,
    ) -> Result<(), crate::LowTuxPdfError> {
        let value = value.into();
        key.into().encode(self.writer)?;
        if value.requires_separator() {
            self.writer.write_all(b" ")?;
        }
        value.encode(self.writer)?;
        Ok(())
    }

    fn start_dictionary(&mut self) -> Result<(), crate::LowTuxPdfError> {
        self.writer.write_all(START_DICTIONARY)?;
        Ok(())
    }

    fn end_dictionary(self) -> Result<(), crate::LowTuxPdfError> {
        self.writer.write_all(END_DICTIONARY)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Object;

    #[test]
    fn test_macro() {
        let dict = dictionary! {
            "Type" => Object::name("Catalog"),
            "Pages" => dictionary!{
                "Type" => Object::name("Pages"),
                "Kids" => vec![
                    dictionary!{
                        "Type" =>Object::name("Page"),
                        "MediaBox" => [0, 0, 612, 792],
                        "Contents" => vec![
                            dictionary!{
                                "Type" => Object::name("Page"),
                                "Length" => 0
                            }
                        ]
                    }
                ]
            }
        };

        println!("{:#?}", dict);
    }
}
