#[allow(dead_code)]
pub fn init_logger() {
    use tracing::{error, info, level_filters::LevelFilter};
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
    static ONCE: std::sync::Once = std::sync::Once::new();
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
#[allow(dead_code)]
pub fn images_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("images")
}
#[allow(dead_code)]
pub fn fonts_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fonts")
}
#[allow(dead_code)]
pub fn destination_dir() -> std::path::PathBuf {
    let folder = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("output");
    if !folder.exists() {
        std::fs::create_dir_all(&folder).unwrap();
    }
    folder
}

#[allow(dead_code)]
pub fn does_end_with_ttf(path: impl AsRef<std::path::Path>) -> bool {
    path.as_ref().extension().map_or(false, |ext| ext == "ttf")
}
