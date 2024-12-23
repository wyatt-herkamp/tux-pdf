use std::sync::Once;

use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn init_logger() {
    static ONCE: Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let stdout_log = tracing_subscriber::fmt::layer().pretty().without_time();
        tracing_subscriber::registry()
            .with(
                stdout_log
                    .with_filter(filter::Targets::new().with_target("tux_pdf", LevelFilter::TRACE)),
            )
            .init();
    });
    info!("Logger initialized");
    error!("This is an error message");
}

pub fn fonts_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fonts")
}

pub fn destination_dir() -> std::path::PathBuf {
    let folder = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("output");
    if !folder.exists() {
        std::fs::create_dir_all(&folder).unwrap();
    }
    folder
}
