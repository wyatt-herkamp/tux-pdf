use rand::{distributions::Alphanumeric, Rng};

/// Returns a string with 32 random characters
pub(crate) fn random_character_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
