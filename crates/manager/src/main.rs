#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use tracing::info;

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();

    let app = App::new()?;
    app.set_setup_done(is_setup_done());
    app.global::<SetupPageLogic>()
        .on_setup(|value| setup(&value));

    app.run()
}

fn is_setup_done() -> bool {
    false
}

fn setup(path: &str) {
    info!("Setup: {path}");
}
