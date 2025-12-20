use fs_extra::copy_items_with_progress;
use fs_extra::dir::CopyOptions;
use fs_extra::dir::TransitProcessResult;
use slint::ComponentHandle as _;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tracing::error;
use tracing::info;

use crate::App;
use crate::Bs2Config;
use crate::SetupPageLogic;
use crate::canonicalize;

pub(crate) fn run_setup(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        app.set_setup_done(false);
        let cs2_path = PathBuf::from(app.global::<Bs2Config>().get_cs2_path().as_str());
        let app = app.as_weak();
        std::thread::spawn(move || {
            let cs2_game_paths_to_copy =
                ["core", "_toolsettings", "thirdpartylegalnotices.txt", "bin"];
            let template_game_paths_to_copy = ["bin", "core", "core_addons"];

            let sources: Vec<_> = cs2_game_paths_to_copy
                .map(|p| cs2_path.join("game").join(p))
                .into_iter()
                .chain(
                    template_game_paths_to_copy.map(|p| Path::new("template").join("game").join(p)),
                )
                .collect();
            let dest = PathBuf::from("game");
            let canon_dest = canonicalize(dest.clone());

            if let Err(e) = fs::create_dir_all(&dest) {
                error!("Failed to create {canon_dest}: {e}");
                slint::invoke_from_event_loop(move || {
                    let app = app.unwrap();
                    app.global::<SetupPageLogic>()
                        .set_toast(format!("Failed to create {canon_dest}: {e}").into());
                    app.global::<SetupPageLogic>().set_toast_error(true);
                })
                .expect("Slint main loop should be running");
                return;
            }
            let content_dirs = ["core", "core_addons"];
            for dir in content_dirs {
                let path = Path::new("content").join(dir);
                let canon = canonicalize(path.clone());
                if let Err(e) = fs::create_dir_all(path) {
                    error!("Failed to create {canon}: {e}");
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>()
                            .set_toast(format!("Failed to create {canon}: {e}").into());
                        app.global::<SetupPageLogic>().set_toast_error(true);
                    })
                    .expect("Slint main loop should be running");
                    return;
                }
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
            let sources = sources
                .iter()
                .map(|s| s.display().to_string())
                .collect::<Vec<_>>();
            match result {
                Ok(written) if written >= total_bytes => {
                    info!("Successfully copied {written} bytes from {sources:?} to {canon_dest}",);
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                        app.set_setup_done(true);
                        app.global::<SetupPageLogic>()
                            .set_toast("BS2 was successfully set up! Switch to the \"Projects\" tab now to manage your projects and launch the Source 2 tools.".into());
                        app.global::<SetupPageLogic>().set_toast_error(false);
                    })
                    .expect("Slint main loop should be running");
                }
                Ok(written) => {
                    error!(
                        "Failed to copy {sources:?} to {canon_dest}: only wrote {written} bytes instead of {total_bytes}"
                    );
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                        app.global::<SetupPageLogic>()
                            .set_toast(format!("Failed to set up BS2: wrote only {written} bytes out of {total_bytes}. Please try again.").into());
                        app.global::<SetupPageLogic>().set_toast_error(true);
                    })
                    .expect("Slint main loop should be running");
                }
                Err(e) => {
                    error!("Failed to copy {sources:?} to {canon_dest}: {e}");
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>().set_copied_thing("".into());
                        app.global::<SetupPageLogic>()
                            .set_toast(format!("Failed to set up BS2: {e}").into());
                        app.global::<SetupPageLogic>().set_toast_error(true);
                    })
                    .expect("Slint main loop should be running");
                }
            };
        });
    }
}

pub(crate) fn is_setup_done() -> bool {
    Path::new("game/bin/win64/engine2.dll").exists()
        && Path::new("game/bin/win64/bs2_launcher.exe").exists()
        && Path::new("content/core_addons").exists()
}

pub(crate) fn update_cs2_state(app: slint::Weak<App>) -> impl FnMut() {
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

pub(crate) fn pick_cs2(app: slint::Weak<App>) -> impl FnMut() {
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
            update_cs2_state(app.as_weak())();
        })
        .expect("Slint event loop should already be initialized");
    }
}
