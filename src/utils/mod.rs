pub mod random;
/// Implements From<&T> for Y where T: Into<Y> and T: Copy
macro_rules! copy_into {
    (
        $type:ty => $target:ty
    ) => {
        impl std::convert::From<&$type> for $target {
            fn from(value: &$type) -> Self {
                (*value).into()
            }
        }
    };
}

use std::borrow::Cow;

pub(crate) use copy_into;

use crate::graphics::PdfOperationType;
pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}
impl<T> IsEmpty for Option<T> {
    fn is_empty(&self) -> bool {
        self.is_none()
    }
}
impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}
impl IsEmpty for &str {
    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }
}
pub trait PartialStruct: IsEmpty {
    type FullStruct: Clone;

    fn merge_with_full<'full>(&self, full: &'full Self::FullStruct)
        -> Cow<'full, Self::FullStruct>;
}

impl<T> PartialStruct for Option<T>
where
    T: PartialStruct,
{
    type FullStruct = T::FullStruct;

    fn merge_with_full<'full>(
        &self,
        full: &'full Self::FullStruct,
    ) -> Cow<'full, Self::FullStruct> {
        match self.as_ref() {
            Some(value) => value.merge_with_full(full),
            None => Cow::Borrowed(full),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum PartailOrFull<T: PartialStruct> {
    Partial(T),
    /// A full value will always cause a full override of values
    Full(T::FullStruct),
}
impl<T: PartialStruct> From<T> for PartailOrFull<T> {
    fn from(full: T) -> Self {
        PartailOrFull::Partial(full)
    }
}

impl<T> Default for PartailOrFull<T>
where
    T: PartialStruct + Default,
{
    fn default() -> Self {
        PartailOrFull::Partial(Default::default())
    }
}
impl<T: PartialStruct> PartailOrFull<T> {
    pub fn merge_with_full<'s>(&'s self, full: &'s T::FullStruct) -> Cow<'s, T::FullStruct> {
        match self {
            PartailOrFull::Partial(partial) => partial.merge_with_full(full),
            PartailOrFull::Full(full) => Cow::Borrowed(full),
        }
    }
}
impl<T: PartialStruct> PdfOperationType for PartailOrFull<T>
where
    T: PdfOperationType,
    T::FullStruct: PdfOperationType,
{
    fn write(
        &self,
        resources: &crate::document::PdfResources,
        writer: &mut crate::graphics::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        match self {
            PartailOrFull::Partial(partial) => partial.write(resources, writer),
            PartailOrFull::Full(full) => full.write(resources, writer),
        }
    }
}
/// Merge two values together
///
/// The Other type always should have priority over the current type
pub trait Merge<Rhs = Self> {
    fn merge(&mut self, other: Rhs);

    fn merge_with_option(&mut self, other: Option<Rhs>) {
        if let Some(other) = other {
            self.merge(other);
        }
    }

    fn merge_into_new(&self, other: Rhs) -> Self
    where
        Self: Clone,
    {
        let mut new = self.clone();
        new.merge(other);
        new
    }
}
