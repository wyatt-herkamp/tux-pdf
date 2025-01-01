use crate::{
    types::{Object, ObjectId, PdfObjectType},
    LowTuxPdfError,
};

pub fn write_object<W>(
    object: Object,
    object_id: ObjectId,
    writer: &mut W,
) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
{
    write_object_type(object_id, object, writer)
}
pub fn write_object_type<W, O>(
    object_id: ObjectId,
    object: O,

    writer: &mut W,
) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
    O: PdfObjectType,
{
    let needs_end_separator = object.requires_end_separator();
    start_object(object_id, object.requires_separator(), writer)?;
    object.encode(writer)?;
    end_object(needs_end_separator, writer)?;
    Ok(())
}
pub fn start_object<W>(
    object_id: ObjectId,
    seperator: bool,
    writer: &mut W,
) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
{
    writeln!(
        writer,
        "{} {} obj",
        object_id.object_number, object_id.generation_number
    )?;
    if seperator {
        writer.write_all(b" ")?;
    }
    Ok(())
}

pub fn end_object<W>(seperator: bool, writer: &mut W) -> Result<(), LowTuxPdfError>
where
    W: std::io::Write,
{
    let end_separator = if seperator { " " } else { "" };
    writeln!(writer, "{}\nendobj", end_separator)?;
    Ok(())
}
