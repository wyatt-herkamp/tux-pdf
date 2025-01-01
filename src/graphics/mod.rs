mod group;
pub mod primitives;
pub use group::*;
use tux_pdf_low::types::Object;
mod ops;
pub mod shapes;
use std::fmt::Debug;
pub mod styles;
pub use ops::*;
pub use styles::*;
pub mod color;
pub mod image;
mod position;
pub mod size;
pub use position::*;
mod other;
pub use other::*;
pub mod text;
pub use text::*;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rotation(pub i64);
impl From<Rotation> for Object {
    fn from(rotation: Rotation) -> Object {
        rotation.0.into()
    }
}
