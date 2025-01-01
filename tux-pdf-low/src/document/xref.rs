use std::{collections::BTreeMap, io::Write};

use crate::{
    types::{
        trailer::{PdfTrailer, XRefPdfTrailer},
        Name, ObjectId, Stream,
    },
    utils::write::write_object_type,
    LowTuxPdfError,
};

#[derive(Debug, Clone)]
pub struct Xref {
    /// Type of Cross-Reference used in the last incremental version.
    /// This method of cross-referencing will also be used when saving the file.
    /// PDFs with Incremental Updates should alway use the same cross-reference type.
    pub cross_reference_type: XrefType,

    /// Entries for indirect object.
    pub entries: BTreeMap<u32, XrefEntry>,

    /// Total number of entries (including free entries), equal to the highest object number plus 1.
    pub size: u32,
}
impl Xref {
    pub fn write_as_stream<W>(
        mut self,
        trailer: PdfTrailer,
        xref_start: usize,
        max_id: u32,
        writer: &mut W,
    ) -> Result<(), LowTuxPdfError>
    where
        W: Write,
    {
        // Increment max_id to account for CRS.
        let max_id = max_id + 1;
        let new_obj_id_for_crs = max_id;
        self.insert(
            new_obj_id_for_crs,
            XrefEntry::Normal {
                offset: xref_start as u32,
                generation: 0,
            },
        );
        let trailer_size = max_id + 1;

        let filter = XRefStreamFilter::None;
        let (stream, stream_length, indexes) = create_xref_stream(&self, filter)?;
        let filter = filter.into_name();

        let trailer = XRefPdfTrailer {
            trailer,
            size: trailer_size,
            w: vec![1, 4, 2],
            index: indexes,
            filter,
            length: stream_length,
        };
        let cross_reference_stream = Stream {
            dictionary: trailer,
            allows_compression: true,
            content: stream,
            start_position: None,
        };

        write_object_type(
            ObjectId::from(new_obj_id_for_crs),
            cross_reference_stream,
            writer,
        )?;

        Ok(())
    }
}
impl Xref {
    pub fn new(size: u32, xref_type: XrefType) -> Xref {
        Xref {
            cross_reference_type: xref_type,
            entries: BTreeMap::new(),
            size,
        }
    }

    pub fn get(&self, id: u32) -> Option<&XrefEntry> {
        self.entries.get(&id)
    }

    pub fn insert(&mut self, id: u32, entry: XrefEntry) {
        self.entries.insert(id, entry);
    }

    /// Combine Xref entries. Only add them if they do not exists already.
    /// Do not replace existing entries.
    pub fn merge(&mut self, xref: Xref) {
        for (id, entry) in xref.entries {
            self.entries.entry(id).or_insert(entry);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear()
    }

    pub fn max_id(&self) -> u32 {
        match self.entries.keys().max() {
            Some(&id) => id,
            None => 0,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum XrefType {
    CrossReferenceStream,
    CrossReferenceTable,
}
#[derive(Debug, Clone)]
pub enum XrefEntry {
    Free, // TODO add generation number
    UnusableFree,
    Normal { offset: u32, generation: u16 },
    Compressed { container: u32, index: u16 },
}
impl XrefEntry {
    pub fn is_normal(&self) -> bool {
        matches!(*self, XrefEntry::Normal { .. })
    }

    pub fn is_compressed(&self) -> bool {
        matches!(*self, XrefEntry::Compressed { .. })
    }

    /// Write Entry in Cross Reference Table.
    pub fn write_xref_entry<W>(&self, file: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        match self {
            XrefEntry::Normal { offset, generation } => {
                writeln!(file, "{:>010} {:>05} n ", offset, generation)?;
            }
            XrefEntry::Compressed {
                container: _,
                index: _,
            } => {
                writeln!(file, "{:>010} {:>05} f ", 0, 65535)?;
            }
            XrefEntry::Free => {
                writeln!(file, "{:>010} {:>05} f ", 0, 0)?;
            }
            XrefEntry::UnusableFree => {
                writeln!(file, "{:>010} {:>05} f ", 0, 65535)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct XrefSection {
    pub starting_id: u32,
    pub entries: Vec<XrefEntry>,
}
impl XrefSection {
    pub fn new(starting_id: u32) -> Self {
        XrefSection {
            starting_id,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: XrefEntry) {
        self.entries.push(entry);
    }

    pub fn add_unusable_free_entry(&mut self) {
        self.add_entry(XrefEntry::UnusableFree);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Write Section in Cross Reference Table.
    pub fn write_xref_section<W>(&self, file: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        if !self.is_empty() {
            // Write section range
            writeln!(file, "{} {}", self.starting_id, self.entries.len())?;
            // Write entries
            for entry in &self.entries {
                entry.write_xref_entry(file)?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum XRefStreamFilter {
    ASCIIHexDecode,
    None,
}
impl XRefStreamFilter {
    pub fn into_name(&self) -> Option<Name> {
        match self {
            XRefStreamFilter::ASCIIHexDecode => Some(Name(b"ASCIIHexDecode".to_vec())),
            XRefStreamFilter::None => None,
        }
    }
}
pub fn create_xref_stream(
    xref: &Xref,
    filter: XRefStreamFilter,
) -> Result<(Vec<u8>, usize, Vec<i64>), LowTuxPdfError> {
    let mut xref_sections = Vec::new();
    let mut xref_section = XrefSection::new(0);

    for obj_id in 1..xref.size + 1 {
        // If section is empty change number of starting id.
        if xref_section.is_empty() {
            xref_section = XrefSection::new(obj_id);
        }
        if let Some(entry) = xref.get(obj_id) {
            xref_section.add_entry(entry.clone());
        } else {
            // Skip over but finish section if not empty
            if !xref_section.is_empty() {
                xref_sections.push(xref_section);
                xref_section = XrefSection::new(obj_id);
            }
        }
    }
    // Print last section
    if !xref_section.is_empty() {
        xref_sections.push(xref_section);
    }

    let mut xref_stream = Vec::new();
    let mut xref_index = Vec::new();

    for section in xref_sections {
        // Add indexes to list
        xref_index.push(section.starting_id as i64);
        xref_index.push(section.entries.len() as i64);
        // Add entries to stream
        let mut obj_id = section.starting_id;
        for entry in section.entries {
            match entry {
                XrefEntry::Free => {
                    // Type 0
                    xref_stream.push(0);
                    xref_stream.extend(obj_id.to_be_bytes());
                    xref_stream.extend(vec![0, 0]); // TODO add generation number
                }
                XrefEntry::UnusableFree => {
                    // Type 0
                    xref_stream.push(0);
                    xref_stream.extend(obj_id.to_be_bytes());
                    xref_stream.extend(65535_u16.to_be_bytes());
                }
                XrefEntry::Normal { offset, generation } => {
                    // Type 1
                    xref_stream.push(1);
                    xref_stream.extend(offset.to_be_bytes());
                    xref_stream.extend(generation.to_be_bytes());
                }
                XrefEntry::Compressed { container, index } => {
                    // Type 2
                    xref_stream.push(2);
                    xref_stream.extend(container.to_be_bytes());
                    xref_stream.extend(index.to_be_bytes());
                }
            }
            obj_id += 1;
        }
    }

    // The end of line character should not be counted, added later.
    let stream_length = xref_stream.len();

    if filter == XRefStreamFilter::ASCIIHexDecode {
        xref_stream = xref_stream
            .iter()
            .flat_map(|c| format!("{:02X}", c).into_bytes())
            .collect::<Vec<u8>>();
    }

    Ok((xref_stream, stream_length, xref_index))
}

pub fn write_xref_section<W>(file: &mut W, xref: &Xref) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
{
    writeln!(file, "xref")?;

    let mut xref_section = XrefSection::new(0);
    // Add first (0) entry
    xref_section.add_unusable_free_entry();

    for obj_id in 1..xref.size {
        // If section is empty change number of starting id.
        if xref_section.is_empty() {
            xref_section = XrefSection::new(obj_id);
        }
        if let Some(entry) = xref.get(obj_id) {
            match *entry {
                XrefEntry::Normal { offset, generation } => {
                    // Add entry
                    xref_section.add_entry(XrefEntry::Normal { offset, generation });
                }
                XrefEntry::Compressed {
                    container: _,
                    index: _,
                } => {
                    xref_section.add_unusable_free_entry();
                }
                XrefEntry::Free => {
                    xref_section.add_entry(XrefEntry::Free);
                }
                XrefEntry::UnusableFree => {
                    xref_section.add_unusable_free_entry();
                }
            }
        } else {
            // Skip over `obj_id`, but finish section if not empty.
            if !xref_section.is_empty() {
                xref_section.write_xref_section(file)?;
                xref_section = XrefSection::new(obj_id);
            }
        }
    }
    // Print last section
    if !xref_section.is_empty() {
        xref_section.write_xref_section(file)?;
    }
    Ok(())
}
