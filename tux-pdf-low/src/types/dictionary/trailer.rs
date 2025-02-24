use crate::types::{Name, NameRef, ObjectId};

use super::{Dictionary, DictionaryType, WritableDictionary};
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfTrailer {
    pub root: Option<ObjectId>,
    pub info: Option<ObjectId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StandardTrailer {
    pub trailer: PdfTrailer,
    pub size: u32,
}
impl DictionaryType for StandardTrailer {
    fn get_type(&self) -> Option<&NameRef> {
        None
    }
    fn from_dictionary(_dict: &mut Dictionary) -> Result<Self, crate::LowTuxPdfError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write_to_dictionary<WD>(self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary,
    {
        if let Some(root) = self.trailer.root {
            dict.write_value(Name::from("Root"), root)?;
        }

        if let Some(info) = self.trailer.info {
            dict.write_value(Name::from("Info"), info)?;
        }
        dict.write_value(Name::from("Size"), self.size)
    }
    fn write_to_dictionary_borrowed<WD>(&self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary,
    {
        self.clone().write_to_dictionary(dict)
    }
}
/// A PDF Trailer that contains the finalized writing info such as Size, Size, W,
pub struct FinalizedPdfTrailer {
    pub trailer: PdfTrailer,
    pub size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XRefPdfTrailer {
    pub trailer: PdfTrailer,
    pub size: u32,
    pub w: Vec<i64>,
    /// Index
    pub index: Vec<i64>,
    pub filter: Option<Name>,
}

impl DictionaryType for XRefPdfTrailer {
    fn get_type(&self) -> Option<&NameRef> {
        Some("XRef".as_ref())
    }
    fn from_dictionary(_dict: &mut Dictionary) -> Result<Self, crate::LowTuxPdfError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write_to_dictionary<WD>(self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary,
    {
        dict.write_value("Type", Name::from("XRef"))?;
        if let Some(root) = self.trailer.root {
            dict.write_value("Root", root)?;
        }

        if let Some(info) = self.trailer.info {
            dict.write_value("Info", info)?;
        }
        dict.write_value("Size", self.size as i64)?;

        dict.write_value("W", self.w)?;
        // TODO: Prevent the conversion to an Object::Array(Vec<Object>)
        dict.write_value("Index", self.index)?;

        if let Some(filter) = self.filter {
            dict.write_value("Filter", filter)?;
        }
        Ok(())
    }
    fn write_to_dictionary_borrowed<WD>(&self, dict: &mut WD) -> Result<(), crate::LowTuxPdfError>
    where
        WD: WritableDictionary,
    {
        self.clone().write_to_dictionary(dict)
    }
}
