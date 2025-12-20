#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::path::PathBuf;

mod config;
mod setup;
use config::Config;

use crate::setup::is_setup_done;
use crate::setup::pick_cs2;
use crate::setup::run_setup;
use crate::setup::update_cs2_state;

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    Config::ensure_exists();

    let app = App::new()?;
    let config = Config::read();
    app.global::<Bs2Config>()
        .set_cs2_path(config.cs2_path.into());

    let app_inner = app.as_weak();
    update_cs2_state(app_inner)();

    let is_setup_done = is_setup_done();
    app.set_setup_done(is_setup_done);
    if is_setup_done {
        app.set_current_item(1);
    }

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>()
        .on_run_setup(run_setup(app_inner));

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>()
        .on_update_cs2_state(update_cs2_state(app_inner));

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>()
        .on_pick_cs2(pick_cs2(app_inner));

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>().on_write_config(move || {
        let app = app_inner.unwrap();
        Config {
            cs2_path: app.global::<Bs2Config>().get_cs2_path().into(),
        }
        .write();
    });

    app.run()
}

fn canonicalize(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    let canon = path.canonicalize().unwrap_or(path);
    let canon = canon.display().to_string();
    canon.trim_start_matches(r"\\?\").to_string()
}
