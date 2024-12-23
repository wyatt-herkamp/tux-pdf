use crate::utils::random::random_character_string;

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct IccProfileId(pub String);
impl Default for IccProfileId {
    fn default() -> Self {
        Self(random_character_string(32))
    }
}
