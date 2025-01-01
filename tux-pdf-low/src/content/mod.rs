use crate::types::{Object, PdfObjectType, PdfType};
#[derive(Debug, Clone, PartialEq)]
pub struct Operation<Obj: PdfObjectType = Object, Op: AsRef<[u8]> = String> {
    pub operation: Op,
    pub arguments: Vec<Obj>,
}
impl Operation {
    pub fn new(operation: impl Into<String>, arguments: Vec<Object>) -> Self {
        Self {
            operation: operation.into(),
            arguments,
        }
    }

    pub fn new_empty(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            arguments: Vec::new(),
        }
    }
}
impl PdfType for Operation {
    fn write<W>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        let Self {
            operation,
            arguments,
        } = self;
        for operand in arguments {
            operand.encode(writer)?;
            writer.write_all(b" ")?;
        }
        writer.write_all(operation.as_ref())?;
        Ok(())
    }
    fn write_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        let Self {
            operation,
            arguments,
        } = self;
        for operand in arguments {
            operand.encode_borrowed(writer)?;
            writer.write_all(b" ")?;
        }
        writer.write_all(operation.as_ref())?;
        Ok(())
    }
}
pub struct Content {
    pub operations: Vec<Operation>,
}

impl PdfType for Content {
    fn write<W>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        let mut first_operation = true;
        for operation in self.operations {
            if first_operation {
                first_operation = false;
            } else {
                writer.write_all(b"\n")?;
            }
            operation.write(writer)?;
        }
        Ok(())
    }

    fn write_borrowed<W>(&self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
    {
        let mut first_operation = true;
        for operation in &self.operations {
            if first_operation {
                first_operation = false;
            } else {
                writer.write_all(b"\n")?;
            }
            operation.write_borrowed(writer)?;
        }
        Ok(())
    }
}
