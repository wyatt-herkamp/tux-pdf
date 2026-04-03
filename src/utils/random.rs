use rand::{RngExt, SeedableRng, distr::Alphanumeric};

/// Returns a string with 32 random characters
pub(crate) fn random_character_string(length: usize) -> String {
    rand::rngs::StdRng::from_rng(&mut rand::rng())
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
