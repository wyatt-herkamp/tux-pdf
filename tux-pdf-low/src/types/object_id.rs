use super::{copy_encode, Dictionary, Object, PdfObjectType};
/// A reference to an object or the object itself
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceOrObject<Obj = Object> {
    Reference(ObjectId),
    Object(Obj),
}
impl<Obj> From<ReferenceOrObject<Obj>> for Object
where
    Obj: Into<Object>,
{
    fn from(reference_or_object: ReferenceOrObject<Obj>) -> Self {
        match reference_or_object {
            ReferenceOrObject::Reference(reference) => Object::Reference(reference),
            ReferenceOrObject::Object(object) => object.into(),
        }
    }
}
impl<Obj> From<ObjectId> for ReferenceOrObject<Obj> {
    fn from(reference: ObjectId) -> Self {
        Self::Reference(reference)
    }
}
impl From<Object> for ReferenceOrObject {
    fn from(object: Object) -> Self {
        match object {
            Object::Reference(reference) => Self::Reference(reference),
            object => Self::Object(object),
        }
    }
}
macro_rules! from_type {
    (
        $(
            $type:ty
        ),*
    ) => {
        $(
            impl From<$type> for ReferenceOrObject<$type> {
                fn from(object: $type) -> Self {
                    Self::Object(object.into())
                }
            }
            impl From<$type> for ReferenceOrObject {
                fn from(object: $type) -> Self {
                    Self::Object(object.into())
                }
            }
        )*
    };
}
from_type!(i64, f32, Dictionary);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    pub(crate) object_number: u32,
    pub(crate) generation_number: u16,
}
impl From<(u32, u16)> for ObjectId {
    fn from((object_number, generation_number): (u32, u16)) -> Self {
        Self {
            object_number,
            generation_number,
        }
    }
}
impl From<u32> for ObjectId {
    fn from(object_number: u32) -> Self {
        Self {
            object_number,
            generation_number: 0,
        }
    }
}
impl ObjectId {
    pub fn new(object_number: u32, generation_number: u16) -> Self {
        Self {
            object_number,
            generation_number,
        }
    }
    pub fn object_number(&self) -> u32 {
        self.object_number
    }
    pub fn increment_generation(&mut self) {
        self.generation_number += 1;
    }
}
impl PdfObjectType for ObjectId {
    fn encode<W>(self, writer: &mut W) -> Result<(), crate::LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        write!(
            writer,
            "{} {} R",
            self.object_number, self.generation_number
        )?;
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
