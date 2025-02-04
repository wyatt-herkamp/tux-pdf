use rand::{distr::Alphanumeric, Rng, SeedableRng};

/// Returns a string with 32 random characters
pub(crate) fn random_character_string(length: usize) -> String {
    rand::rngs::StdRng::from_os_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
