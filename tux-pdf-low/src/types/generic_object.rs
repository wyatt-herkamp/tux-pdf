use crate::LowTuxPdfError;

use super::{Dictionary, Name, NameRef, Null, ObjectId, PdfObjectType, PdfString, Stream};
/// An Object is just an enum that is used to represent the different types of objects in a PDF file.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    /// The null object see [Null]
    Null,
    /// A boolean object
    Boolean(bool),
    /// An integer object
    Integer(i64),
    /// A floating point number object
    Real(f32),
    /// A string object
    ///
    /// See [PdfString]
    String(PdfString),
    /// A dictionary object
    ///
    /// See [Dictionary]
    Dictionary(Dictionary),
    /// An array object
    Array(Vec<Object>),
    /// A name object
    /// See [Name]
    Name(Name),
    /// A stream object
    ///
    /// See [Stream]
    Stream(Stream),
    /// A reference object
    Reference(ObjectId),
}
impl Object {
    /// Creates a new string object from a borrowed byte slice
    ///
    /// ```rust
    ///  use tux_pdf_low::types::Object;
    /// let obj = Object::literal("Hello, World!");
    ///
    /// assert_eq!(obj.as_string().unwrap().as_slice(), b"Hello, World!");
    /// ```
    pub fn literal<B: AsRef<[u8]> + ?Sized>(b: &B) -> Self {
        Object::String(PdfString::literal(b))
    }
    /// Creates a new string object from an owned byte vector
    pub fn string_literal_owned(value: impl Into<Vec<u8>>) -> Self {
        Object::String(PdfString::literal_owned(value))
    }
    /// Creates a new string object from a borrowed byte slice
    pub fn name(name: impl Into<Name>) -> Self {
        Object::Name(name.into())
    }
}

macro_rules! basic_from {
    (
        $(
            $type:ty => $variant:ident,
        )*
    ) => {
        $(
            impl From<$type> for Object {
                fn from(value: $type) -> Self {
                    Object::$variant(value)
                }
            }
        )*
    };
    (
        $(
            $type:ty as $as_type:ty => $variant:ident
        ),*
    ) => {
        $(
            impl From<$type> for Object {
                fn from(value: $type) -> Self {
                    Object::$variant(value as $as_type)
                }
            }
        )*
    };
}
basic_from! {
    i8 as i64 => Integer,
    i16 as i64 => Integer,
    i32 as i64 => Integer,
    i64 as i64 => Integer,
    u8 as i64 => Integer,
    u16 as i64 => Integer,
    u32 as i64 => Integer,
    u64 as i64 => Integer,
    f32 as f32 => Real,
    f64 as f32 => Real
}
basic_from! {
    bool => Boolean,
    PdfString => String,
    Dictionary => Dictionary,
    Name => Name,
    Stream => Stream,
    ObjectId => Reference,
}
impl<T, const N: usize> From<[T; N]> for Object
where
    T: Into<Object> + Copy,
{
    fn from(value: [T; N]) -> Self {
        Object::Array(value.iter().copied().map(Into::into).collect())
    }
}
impl<T> From<Vec<T>> for Object
where
    T: Into<Object>,
{
    fn from(value: Vec<T>) -> Self {
        Object::Array(value.into_iter().map(Into::into).collect())
    }
}
impl From<&NameRef> for Object {
    fn from(value: &NameRef) -> Self {
        Object::Name(value.to_owned())
    }
}
macro_rules! as_type {
    (
        $(
           $fn_name:ident => $variant:ident -> $type:ty
        ),*
    ) => {

        $(
            pub fn $fn_name(&self) -> Option<&$type> {
                match self {
                    Object::$variant(v) => Some(v),
                    _ => None,
                }
            }
        )*
    };
    (
        $(
           $fn_name:ident => $variant:ident -> *$type:ty
        ),*
    ) => {

        $(
            pub fn $fn_name(&self) -> Option<$type> {
                match self {
                    Object::$variant(v) => Some(*v),
                    _ => None,
                }
            }
        )*
    };
}
impl Object {
    /// Returns the object as a dictionary if it is a dictionary or a stream.
    pub fn as_dictionary_or_stream_dictionary(&self) -> Option<&Dictionary> {
        match self {
            Object::Dictionary(d) => Some(d),
            Object::Stream(s) => Some(&s.dictionary),
            _ => None,
        }
    }
    as_type!(
        as_string => String -> PdfString,
        as_name => Name -> Name,
        as_array => Array -> Vec<Object>,
        as_dictionary => Dictionary -> Dictionary,
        as_stream => Stream -> Stream
    );
    as_type!(
        as_reference => Reference -> ObjectId,
        as_boolean => Boolean -> bool,
        as_integer => Integer -> i64,
        as_real => Real -> f32
    );
}

impl PdfObjectType for Object {
    fn encode<W>(self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
        Self: Sized,
    {
        match self {
            Object::Null => Null.encode(writer),
            Object::Boolean(b) => b.encode(writer),
            Object::Integer(i) => i.encode(writer),
            Object::Real(f) => f.encode(writer),
            Object::String(s) => s.encode(writer),
            Object::Name(n) => n.encode(writer),
            Object::Dictionary(d) => d.encode(writer),
            Object::Stream(s) => s.encode(writer),
            Object::Array(a) => a.encode(writer),
            Object::Reference(r) => r.encode(writer),
        }
    }

    fn encode_borrowed<W>(&self, writer: &mut W) -> Result<(), LowTuxPdfError>
    where
        W: std::io::Write,
    {
        match self {
            Object::Null => Null.encode(writer),
            Object::Boolean(b) => b.encode_borrowed(writer),
            Object::Integer(i) => i.encode_borrowed(writer),
            Object::Real(f) => f.encode_borrowed(writer),
            Object::String(s) => s.encode_borrowed(writer),
            Object::Name(n) => n.encode_borrowed(writer),
            Object::Dictionary(d) => d.encode_borrowed(writer),
            Object::Stream(s) => s.encode_borrowed(writer),
            Object::Array(a) => a.encode_borrowed(writer),
            Object::Reference(r) => r.encode(writer),
        }
    }
    fn requires_end_separator(&self) -> bool {
        match self {
            Object::Dictionary(v) => v.requires_end_separator(),
            Object::String(v) => v.requires_end_separator(),
            Object::Name(v) => v.requires_end_separator(),
            Object::Null => Null.requires_end_separator(),
            Object::Boolean(v) => v.requires_end_separator(),
            Object::Integer(v) => v.requires_end_separator(),
            Object::Real(v) => v.requires_end_separator(),
            Object::Stream(v) => v.requires_end_separator(),
            Object::Array(v) => v.requires_end_separator(),
            Object::Reference(v) => v.requires_end_separator(),
        }
    }

    fn requires_separator(&self) -> bool {
        match self {
            Object::Dictionary(v) => v.requires_separator(),
            Object::String(v) => v.requires_separator(),
            Object::Name(v) => v.requires_separator(),
            Object::Null => Null.requires_separator(),
            Object::Boolean(v) => v.requires_separator(),
            Object::Integer(v) => v.requires_separator(),
            Object::Real(v) => v.requires_separator(),
            Object::Stream(v) => v.requires_separator(),
            Object::Array(v) => v.requires_separator(),
            Object::Reference(v) => v.requires_separator(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Object::Dictionary(v) => v.type_name(),
            Object::String(v) => v.type_name(),
            Object::Name(v) => v.type_name(),
            Object::Null => Null.type_name(),
            Object::Boolean(v) => v.type_name(),
            Object::Integer(v) => v.type_name(),
            Object::Real(v) => v.type_name(),
            Object::Stream(v) => v.type_name(),
            Object::Array(v) => v.type_name(),
            Object::Reference(v) => v.type_name(),
        }
    }
}
