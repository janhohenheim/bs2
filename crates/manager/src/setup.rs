use fs_extra::copy_items_with_progress;
use fs_extra::dir::CopyOptions;
use fs_extra::dir::TransitProcessResult;
use slint::ComponentHandle as _;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use tracing::error;
use tracing::info;

use crate::App;
use crate::Bs2Config;
use crate::Cs2State;
use crate::SetupPageLogic;
use crate::canonicalize;
use crate::config::Config;
use crate::pick_path;
use crate::working_dir;

pub(super) fn plugin(app: &App) {
    let config = Config::read();
    app.global::<SetupPageLogic>()
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
    app.global::<SetupPageLogic>().on_pick_cs2(pick_path(
        app_inner,
        "Choose Counter Strike 2 install path",
        |app, path| {
            app.global::<SetupPageLogic>().set_cs2_path(path);
            update_cs2_state(app.as_weak())();
        },
    ));
}

pub(crate) fn run_setup(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        if !working_dir().join("template").exists() {
            app.global::<SetupPageLogic>()
                .set_toast((true, "Cannot set up BS2: the template/ directory is missing. Try completely uninstalling and reinstalling BS2.".to_string().into()));
            return;
        }
        if is_locked(
            working_dir()
                .join("game")
                .join("bin")
                .join("win64")
                .join("engine2.dll"),
        ) {
            app.global::<SetupPageLogic>()
                .set_toast((true, "Cannot set up BS2 while the Source 2 Tools are in use. Please close all of them first.".to_string().into()));
            return;
        }
        app.set_setup_done(false);
        app.global::<SetupPageLogic>().set_toast((false, "".into()));
        let cs2_path = PathBuf::from(app.global::<Bs2Config>().get_cs2_path().as_str());
        let app = app.as_weak();
        std::thread::spawn(move || {
            let cs2_game_paths_to_copy = ["core", "thirdpartylegalnotices.txt", "bin"];
            let template_game_paths_to_copy = ["bin", "core", "core_addons"];

            let sources: Vec<_> = cs2_game_paths_to_copy
                .map(|p| cs2_path.join("game").join(p))
                .into_iter()
                .chain(
                    template_game_paths_to_copy
                        .map(|p| working_dir().join("template").join("game").join(p)),
                )
                .collect();
            let dest = working_dir().join("game");
            let canon_dest = canonicalize(dest.clone());

            if let Err(e) = fs::create_dir_all(&dest) {
                error!("Failed to create {canon_dest}: {e}");
                slint::invoke_from_event_loop(move || {
                    let app = app.unwrap();
                    app.global::<SetupPageLogic>()
                        .set_toast((true, format!("Failed to create {canon_dest}: {e}").into()));
                })
                .expect("Slint main loop should be running");
                return;
            }
            let content_dirs = ["core", "core_addons"];
            for dir in content_dirs {
                let path = working_dir().join("content").join(dir);
                let canon = canonicalize(path.clone());
                if let Err(e) = fs::create_dir_all(path) {
                    error!("Failed to create {canon}: {e}");
                    slint::invoke_from_event_loop(move || {
                        let app = app.unwrap();
                        app.global::<SetupPageLogic>()
                            .set_toast((true, format!("Failed to create {canon}: {e}").into()));
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
                        app.global::<SetupPageLogic>().set_copy_progress((
                            path.to_string_lossy().to_string().into(),
                            progress as f32,
                        ));
                    })
                    .expect("Slint main loop should be running");
                    TransitProcessResult::ContinueOrAbort
                },
            );
            let sources = sources
                .iter()
                .map(|s| s.display().to_string())
                .collect::<Vec<_>>();
            let app_inner = app.clone();
            match result {
                Ok(written) if written >= total_bytes => {
                    info!("Successfully copied {written} bytes from {sources:?} to {canon_dest}",);
                    slint::invoke_from_event_loop(move || {
                        let app = app_inner.unwrap();
                        app.global::<SetupPageLogic>()
                            .set_copy_progress(("renaming files...".into(), 0.0));
                    })
                    .expect("Slint main loop should be running");
                }
                Ok(written) => {
                    error!(
                        "Failed to copy {sources:?} to {canon_dest}: only wrote {written} bytes instead of {total_bytes}"
                    );
                    slint::invoke_from_event_loop(move || {
                        let app = app_inner.unwrap();
                        app.global::<SetupPageLogic>().set_copy_progress(("".into(), 0.0));
                        app.global::<SetupPageLogic>()
                            .set_toast((true,format!("Failed to set up BS2: wrote only {written} bytes out of {total_bytes}. Please try again.").into()));
                    })
                    .expect("Slint main loop should be running");
                    return;
                }
                Err(e) => {
                    error!("Failed to copy {sources:?} to {canon_dest}: {e}");
                    slint::invoke_from_event_loop(move || {
                        let app = app_inner.unwrap();
                        app.global::<SetupPageLogic>()
                            .set_copy_progress(("".into(), 0.0));
                        app.global::<SetupPageLogic>().set_toast((
                            true,
                            format!("Failed to set up BS2: {e} ({:?})", e.kind).into(),
                        ));
                    })
                    .expect("Slint main loop should be running");
                    return;
                }
            };
            let bin = dest.join("bin").join("win64");
            let app_inner = app.clone();
            let src = bin.join("resourcecompiler.exe");
            let dest = bin.join("resourcecompiler_inner.exe");
            if let Err(e) = fs::rename(&src, &dest) {
                slint::invoke_from_event_loop(move || {
                    let app = app_inner.unwrap();
                    app.global::<SetupPageLogic>().set_toast((
                        true,
                        format!(
                            "Failed to rename {} to {}: {e} ({:?})",
                            src.display(),
                            dest.display(),
                            e.kind()
                        )
                        .into(),
                    ));
                })
                .expect("Slint main loop should be running");
                return;
            }
            let app_inner = app.clone();
            let src = bin.join("bs2_resourcecompiler.exe");
            let dest = bin.join("resourcecompilerr.exe");
            if let Err(e) = fs::rename(&src, &dest) {
                slint::invoke_from_event_loop(move || {
                    let app = app_inner.unwrap();
                    app.global::<SetupPageLogic>().set_toast((
                        true,
                        format!(
                            "Failed to rename {} to {}: {e} ({:?})",
                            src.display(),
                            dest.display(),
                            e.kind()
                        )
                        .into(),
                    ));
                })
                .expect("Slint main loop should be running");
                return;
            }

            slint::invoke_from_event_loop(move || {
                        let app = app_inner.unwrap();
                        app.global::<SetupPageLogic>().set_copy_progress(("".into(), 0.0));
                        app.set_setup_done(true);
                        app.global::<SetupPageLogic>()
                            .set_toast((false, "BS2 was successfully set up! Switch to the \"Projects\" tab now to manage your projects and launch the Source 2 tools.".into()));
                    })
                    .expect("Slint main loop should be running");
        });
    }
}

pub(crate) fn is_setup_done() -> bool {
    working_dir().join("game/bin/win64/engine2.dll").exists()
        && working_dir()
            .join("game/bin/win64/bs2_launcher.exe")
            .exists()
        && working_dir().join("content/core_addons").exists()
}

pub(crate) fn update_cs2_state(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        let cs2_path = app.global::<SetupPageLogic>().get_cs2_path();
        let engine_dll = PathBuf::from(cs2_path.as_str())
            .join("game")
            .join("bin")
            .join("win64")
            .join("engine2.dll");
        let engine_exists = engine_dll.exists();
        if !engine_exists {
            app.global::<SetupPageLogic>().set_cs2_state(Cs2State::None);
            return;
        }
        let workshop_fgd = PathBuf::from(cs2_path.as_str())
            .join("game")
            .join("core")
            .join("base.fgd");
        if !workshop_fgd.exists() {
            app.global::<SetupPageLogic>()
                .set_cs2_state(Cs2State::GameOnly);
        } else {
            app.global::<SetupPageLogic>().set_cs2_state(Cs2State::Good);
        }
    }
}

fn is_locked(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if !path.exists() {
        return false;
    }
    OpenOptions::new()
        .write(true)
        .create(false)
        .open(path)
        .is_err()
}
