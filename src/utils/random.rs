use rand::{distributions::Alphanumeric, Rng};

use crate::graphics::color::{Color, Rgb};

/// Returns a string with 32 random characters
pub(crate) fn random_character_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub(crate) fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    Color::Rgb(Rgb {
        r: rng.gen_range(0.0..1.0),
        g: rng.gen_range(0.0..1.0),
        b: rng.gen_range(0.0..1.0),
        icc_profile: None,
    })
}
