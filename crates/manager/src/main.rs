#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::path::Path;
use std::path::PathBuf;
use tracing::info;

mod config;
use config::Config;

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    Config::ensure_exists();

    let app = App::new()?;
    let config = Config::read();
    app.global::<Bs2Config>()
        .set_cs2_path(config.cs2_path.into());

    let app_inner = app.as_weak();
    update_cs2_state(app_inner)();

    app.set_setup_done(is_setup_done());

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

fn is_setup_done() -> bool {
    Path::new("game/bin/win64/engine2.dll").exists()
}

fn run_setup(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        let path = app.global::<Bs2Config>().get_cs2_path();
        info!("Setup {path}");
    }
}

fn update_cs2_state(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        let cs2_path = app.global::<Bs2Config>().get_cs2_path();
        let engine_dll = PathBuf::from(cs2_path.as_str())
            .join("game")
            .join("bin")
            .join("win64")
            .join("engine2.dll");
        let engine_exists = engine_dll.exists();
        if !engine_exists {
            info!("noo {}", engine_dll.display());
            app.global::<SetupPageLogic>().set_cs2_state("none".into());
            return;
        }
        let workshop_fgd = PathBuf::from(cs2_path.as_str())
            .join("game")
            .join("core")
            .join("base.fgd");
        if !workshop_fgd.exists() {
            app.global::<SetupPageLogic>()
                .set_cs2_state("only-game".into());
        } else {
            app.global::<SetupPageLogic>().set_cs2_state("good".into());
        }
    }
}

fn pick_cs2(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        slint::spawn_local(async move {
            let current = app.global::<Bs2Config>().get_cs2_path();
            let path = rfd::AsyncFileDialog::new()
                .set_parent(&app.window().window_handle())
                .set_can_create_directories(false)
                .set_directory(current)
                .set_title("Choose Counter Strike 2 install path")
                .pick_folder()
                .await;
            let Some(path) = path else {
                return;
            };
            app.global::<Bs2Config>()
                .set_cs2_path(path.path().to_string_lossy().to_string().into());
            update_cs2_state(app.as_weak())()
        })
        .expect("Slint event loop should already be initialized");
    }
}
