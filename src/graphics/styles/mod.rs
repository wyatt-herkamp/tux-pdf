mod margin;
mod padding;
pub use margin::*;
pub use padding::*;
use std::ops::Add;

pub(crate) fn add_two_optional<U>(a: Option<U>, b: Option<U>) -> Option<U>
where
    U: Add<Output = U> + Copy,
{
    match (a, b) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
