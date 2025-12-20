#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use fs_extra::copy_items_with_progress;
use fs_extra::dir::CopyOptions;
use fs_extra::dir::TransitProcessResult;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tracing::error;
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
        let cs2_path = PathBuf::from(app.global::<Bs2Config>().get_cs2_path().as_str());
        let app = app.as_weak();
        std::thread::spawn(move || {
            let paths_to_copy = ["core", "_toolsettings", "thirdpartylegalnotices.txt", "bin"];

            let sources = paths_to_copy.map(|p| cs2_path.join("game").join(p));
            let dest = PathBuf::from("game");
            let canon_dest = dest.canonicalize().unwrap_or_else(|_| dest.clone());
            let canon_dest = canon_dest.display();
            let canon_dest = canon_dest.to_string();
            let canon_dest = canon_dest.trim_start_matches("\\\\?\\");

            if let Err(e) = fs::create_dir_all(&dest) {
                error!("Failed to create {canon_dest}: {e:?}");
            }
            let app_inner = app.clone();
            let mut total_bytes = 0;
            let result = copy_items_with_progress(
                &sources,
                &dest,
                &CopyOptions {
                    overwrite: true,
                    ..Default::default()
                },
                |process| {
                    let name = if process.file_name.is_empty() {
                        &process.dir_name
                    } else {
                        &process.file_name
                    };
                    let path = cs2_path.join("game").join(name);
                    total_bytes = process.total_bytes;
                    let progress = process.copied_bytes as f64 / process.total_bytes as f64;
                    info!("{}: {}", path.display(), progress);
                    let app = app_inner.clone();
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>()
                            .set_copied_thing(path.to_string_lossy().to_string().into());
                        app.global::<SetupPageLogic>().set_progress(progress as f32);
                    })
                    .expect("Slint main loop should be running");
                    TransitProcessResult::ContinueOrAbort
                },
            );
            match result {
                Ok(written) if written >= total_bytes => {
                    info!(
                        "Successfully copied {written} bytes from {:?} to {canon_dest}",
                        sources.map(|s| s.display().to_string()),
                    );
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                        app.set_setup_done(true);
                        app.set_current_item(1);
                    })
                    .expect("Slint main loop should be running");
                }
                Ok(written) => {
                    error!(
                        "Failed to copy {:?} to {canon_dest}: only wrote {written} bytes instead of {total_bytes}",
                        sources.map(|s| s.display().to_string()),
                    );
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                    })
                    .expect("Slint main loop should be running");
                }
                Err(e) => {
                    error!(
                        "Failed to copy {:?} to {canon_dest}: {e:?}",
                        sources.map(|s| s.display().to_string()),
                    );
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                    })
                    .expect("Slint main loop should be running");
                }
            };
        });
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
