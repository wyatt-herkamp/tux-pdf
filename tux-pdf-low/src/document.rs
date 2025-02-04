use std::{collections::BTreeMap, io::Write};
pub mod xref;

use tracing::trace;
use xref::{Xref, XrefEntry, XrefType};

use crate::{
    types::{
        trailer::{PdfTrailer, StandardTrailer},
        Dictionary, DictionaryIoWriter, DictionaryType, Object, ObjectId, PdfType, PdfVersion,
        WritableDictionary,
    },
    utils::write::write_object,
};
#[derive(Debug)]
pub struct PdfDocumentWriter {
    pub version: PdfVersion,
    pub trailer: PdfTrailer,
    objects: BTreeMap<ObjectId, Object>,
    pub cross_reference_type: XrefType,
    max_id: u32,
}
impl Default for PdfDocumentWriter {
    fn default() -> Self {
        Self {
            version: PdfVersion::default(),
            trailer: PdfTrailer::default(),
            objects: BTreeMap::new(),
            cross_reference_type: XrefType::CrossReferenceTable,
            max_id: 0,
        }
    }
}

impl PdfDocumentWriter {
    pub fn new(version: impl Into<PdfVersion>) -> PdfDocumentWriter {
        PdfDocumentWriter {
            version: version.into(),
            ..Default::default()
        }
    }
    pub fn save<W: Write>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError> {
        let mut xref = Xref::new(self.max_id + 1, self.cross_reference_type);

        let mut writer = crate::utils::CountingWriter::new(writer);
        self.version.write(&mut writer)?;
        for (object_id, object) in self.objects {
            if object
                .as_dictionary_or_stream_dictionary()
                .map(should_skip_dictionary)
                .unwrap_or_default()
            {
                trace!(?object_id, ?object, "Skipping item");
                continue;
            }
            trace!(?object_id, ?object, "Writing object");
            let offset = writer.count() as u32;
            xref.insert(
                object_id.object_number(),
                XrefEntry::Normal {
                    offset,
                    generation: object_id.generation_number,
                },
            );
            write_object(object, object_id, &mut writer)?;
        }
        let xref_start = writer.count();

        match self.cross_reference_type {
            xref::XrefType::CrossReferenceStream => {
                xref::write_xref_section(&mut writer, &xref)?;
                let trailer = StandardTrailer {
                    trailer: self.trailer,
                    size: self.max_id + 1,
                };
                writer.write_all(b"trailer\n")?;
                let mut dictionary_writer = DictionaryIoWriter::from(&mut writer);
                dictionary_writer.start_dictionary()?;
                trailer.write_to_dictionary(&mut dictionary_writer)?;
                dictionary_writer.end_dictionary()?;
            }
            xref::XrefType::CrossReferenceTable => {
                xref.write_as_stream(self.trailer, xref_start, self.max_id, &mut writer)?;
            }
        }
        write!(writer, "\nstartxref\n{}\n%%EOF", xref_start)?;

        Ok(())
    }
    pub fn next_object_id(&mut self) -> ObjectId {
        self.max_id += 1;
        ObjectId::from(self.max_id)
    }
    /// Add an object to the document. Returning its object id.
    pub fn add_object(&mut self, object: impl Into<Object>) -> ObjectId {
        let id = self.next_object_id();
        self.objects.insert(id, object.into());
        id
    }
    /// Add an object to the document with a specific object id.
    pub fn set_object(&mut self, id: ObjectId, object: impl Into<Object>) {
        self.objects.insert(id, object.into());
    }

    pub fn get_object(&self, id: &ObjectId) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn remove_object(&mut self, id: &ObjectId) -> Option<Object> {
        self.objects.remove(id)
    }
    /// Find all objects with the given object number.
    pub fn find_object_by_object_number(&self, object_number: u32) -> Vec<&Object> {
        self.objects
            .iter()
            .filter_map(|(id, object)| {
                if id.object_number() == object_number {
                    Some(object)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn should_skip_dictionary(dict: &Dictionary) -> bool {
    match dict.dictionary_type() {
        Ok(Some(name)) => matches!(name.as_slice(), b"ObjStm" | b"XRef" | b"Linearized"),
        Ok(None) => false,
        Err(_) => false,
    }
}
