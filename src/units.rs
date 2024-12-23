/*! Units of measurement

| Unit Type | Name       | Note                        |
|-----------|------------|-----------------------------|
| [Pt]        | Point      | Standard PDF Unit           |
| [Mm]        | millimeter |                             |
| [Px]        | pixels     | Requires DPI for conversion |

*/

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::FpCategory;

macro_rules! from_inner {
    ($type:ident($inner:ty)) => {
        impl From<$inner> for $type {
            fn from(val: $inner) -> Self {
                $type(val)
            }
        }
    };
}
macro_rules! serde_transparent {
    (
        $type:ident($inner:ty)
    ) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let inner = <$inner>::deserialize(deserializer)?;
                Ok($type(inner))
            }
        }
    };
}
macro_rules! impl_partialeq {
    ($t:ty) => {
        impl PartialEq for $t {
            // custom compare function because of floating point inaccuracy
            fn eq(&self, other: &$t) -> bool {
                if (self.0.classify() == FpCategory::Zero
                    || self.0.classify() == FpCategory::Normal)
                    && (other.0.classify() == FpCategory::Zero
                        || other.0.classify() == FpCategory::Normal)
                {
                    // four floating point numbers have to match
                    (self.0 * 1000.0).round() == (other.0 * 1000.0).round()
                } else {
                    false
                }
            }
        }
    };
}
macro_rules! into_lo_object {
    (
        $type:ident
    ) => {
        impl From<$type> for lopdf::Object {
            fn from(val: $type) -> Self {
                val.0.into()
            }
        }
    };
}

macro_rules! self_math {
    ($type:ident) => {
        impl $type {
            pub fn min(self, other: Self) -> Self {
                Self(self.0.min(other.0))
            }
            pub fn max(self, other: Self) -> Self {
                Self(self.0.max(other.0))
            }
        }
        impl std::ops::Add for $type {
            type Output = $type;
            fn add(self, other: $type) -> $type {
                $type(self.0 + other.0)
            }
        }
        impl std::ops::Sub for $type {
            type Output = $type;
            fn sub(self, other: $type) -> $type {
                $type(self.0 - other.0)
            }
        }
        impl std::ops::Mul for $type {
            type Output = $type;
            fn mul(self, other: $type) -> $type {
                $type(self.0 * other.0)
            }
        }
        impl std::ops::Div for $type {
            type Output = $type;
            fn div(self, other: $type) -> $type {
                $type(self.0 / other.0)
            }
        }
        impl std::ops::AddAssign for $type {
            fn add_assign(&mut self, other: $type) {
                self.0 += other.0;
            }
        }
        impl std::ops::SubAssign for $type {
            fn sub_assign(&mut self, other: $type) {
                self.0 -= other.0;
            }
        }
        impl std::ops::MulAssign for $type {
            fn mul_assign(&mut self, other: $type) {
                self.0 *= other.0;
            }
        }
        impl std::ops::DivAssign for $type {
            fn div_assign(&mut self, other: $type) {
                self.0 /= other.0;
            }
        }
        impl std::iter::Sum for $type {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::default(), |acc, x| acc + x)
            }
        }
    };
    ($type:ident($inner:ty)) => {
        self_math!($type);

        impl std::ops::Add<$inner> for $type {
            type Output = $type;
            fn add(self, other: $inner) -> $type {
                $type(self.0 + other)
            }
        }
        impl std::ops::Sub<$inner> for $type {
            type Output = $type;
            fn sub(self, other: $inner) -> $type {
                $type(self.0 - other)
            }
        }
        impl std::ops::Mul<$inner> for $type {
            type Output = $type;
            fn mul(self, other: $inner) -> $type {
                $type(self.0 * other)
            }
        }
        impl std::ops::Div<$inner> for $type {
            type Output = $type;
            fn div(self, other: $inner) -> $type {
                $type(self.0 / other)
            }
        }
        impl std::ops::AddAssign<$inner> for $type {
            fn add_assign(&mut self, other: $inner) {
                self.0 += other;
            }
        }
        impl std::ops::SubAssign<$inner> for $type {
            fn sub_assign(&mut self, other: $inner) {
                self.0 -= other;
            }
        }
        impl std::ops::MulAssign<$inner> for $type {
            fn mul_assign(&mut self, other: $inner) {
                self.0 *= other;
            }
        }
        impl std::ops::DivAssign<$inner> for $type {
            fn div_assign(&mut self, other: $inner) {
                self.0 /= other;
            }
        }
    };
}
/// Scale in millimeter
#[derive(Debug, Default, Copy, Clone, PartialOrd)]
pub struct Mm(pub f32);
serde_transparent!(Mm(f32));
impl_partialeq!(Mm);
self_math!(Mm(f32));
into_lo_object!(Mm);
from_inner!(Mm(f32));
impl From<Mm> for Pt {
    fn from(mm: Mm) -> Self {
        Pt(mm.0 * 2.834_646_f32)
    }
}
/// Scale in point
#[derive(Debug, Default, Copy, Clone, PartialOrd)]
pub struct Pt(pub f32);
self_math!(Pt(f32));
impl_partialeq!(Pt);
serde_transparent!(Pt(f32));
into_lo_object!(Pt);
from_inner!(Pt(f32));
impl From<Pt> for Mm {
    fn from(pt: Pt) -> Self {
        Mm(pt.0 * 0.352_778_f32)
    }
}
/// Scale in pixels
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Px(pub i64);
self_math!(Px(i64));
serde_transparent!(Px(i64));
into_lo_object!(Px);
from_inner!(Px(i64));

pub trait UnitType {
    fn mm(&self) -> Mm;
    fn pt(&self) -> Pt;
    fn px(&self) -> Px;
}
impl UnitType for Pt {
    fn mm(&self) -> Mm {
        Mm(self.0 * 0.352_778_f32)
    }
    fn pt(&self) -> Pt {
        *self
    }
    fn px(&self) -> Px {
        Px((self.0 * 1.333_333_4) as i64)
    }
}

impl UnitType for Mm {
    fn mm(&self) -> Mm {
        *self
    }
    fn pt(&self) -> Pt {
        Pt(self.0 / 2.834_646_f32)
    }
    fn px(&self) -> Px {
        Px((self.0) as i64)
    }
}

impl UnitType for f32 {
    fn mm(&self) -> Mm {
        Mm(*self)
    }
    fn pt(&self) -> Pt {
        Pt(*self)
    }
    fn px(&self) -> Px {
        Px(*self as i64)
    }
}
macro_rules! unit_type_core_types {
    (
        $(
            $type:ty
        ),*
    ) => {
        $(
            impl UnitType for $type {
                fn mm(&self) -> Mm {
                    Mm(*self as f32)
                }
                fn pt(&self) -> Pt {
                    Pt(*self as f32)
                }
                fn px(&self) -> Px {
                    Px(*self as i64)
                }
            }
        )*
    };
}
// Following implementations just use as conversion and assumes what is passed doesn't need scaling
// If it needs scaling convert it to the appropriate type first
unit_type_core_types!(i64, i32, i16, i8, u64, u32, u16, u8);
