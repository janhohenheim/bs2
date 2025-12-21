#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::env;
use std::fs;
use std::iter;
use std::os::windows::process::CommandExt as _;
use std::path::PathBuf;
use std::process::Command;

mod config;
mod setup;
use config::Config;
use slint::ModelRc;
use slint::SharedString;

use crate::config::Project;
use crate::setup::is_setup_done;
use crate::setup::run_setup;
use crate::setup::update_cs2_state;

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    Config::ensure_exists();

    let app = App::new()?;
    let config = Config::read();
    app.global::<Bs2Config>()
        .set_cs2_path(config.cs2_path.clone().into());

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

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>().on_write_config(move || {
        let app = app_inner.unwrap();
        Config {
            cs2_path: app.global::<Bs2Config>().get_cs2_path().into(),
            ..Config::read()
        }
        .write();
    });

    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_update_project_selection(move || {
            let app = app_inner.unwrap();
            app.global::<ProjectsPageLogic>()
                .set_project_selection(ModelRc::from(
                    iter::once("➕︎ Create New Project")
                        .chain(Config::read().projects.iter().map(|p| p.name.as_str()))
                        .map(std::convert::Into::into)
                        .collect::<Vec<_>>()
                        .as_ref(),
                ));
        });

    app.global::<ProjectsPageLogic>()
        .invoke_update_project_selection();
    app.global::<ProjectsPageLogic>()
        .on_read_project(move |name| {
            let project = Config::read()
                .projects
                .iter()
                .find(|p| p.name.as_str() == name.as_str())
                .cloned()
                .unwrap_or_default();
            ProjectDef {
                name: project.name.into(),
                path: project.path.into(),
            }
        });
    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>().on_pick_project(pick_path(
        app_inner,
        "Choose project path",
        |app, path| {
            app.global::<ProjectsPageLogic>().set_new_project_path(path);
        },
    ));
    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_create_project(move || {
            let app = app_inner.unwrap();
            let mut config = Config::read();
            let name: String = app
                .global::<ProjectsPageLogic>()
                .get_new_project_name()
                .into();
            config.projects.push(Project {
                name: name.clone(),
                path: app
                    .global::<ProjectsPageLogic>()
                    .get_new_project_path()
                    .into(),
            });
            fs::create_dir_all(
                working_dir()
                    .join("game")
                    .join("core_addons")
                    .join(name.as_str()),
            )
            .expect("Oof");
            fs::create_dir_all(
                working_dir()
                    .join("content")
                    .join("core_addons")
                    .join(name.as_str()),
            )
            .expect("Oof");
            config.write();
        });
    app.global::<ProjectsPageLogic>()
        .on_launch_tools(move |name| {
            // Source: https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

            let dir = working_dir().join("game").join("bin").join("win64");
            let path = dir.join("bs2_launcher.exe");
            std::thread::spawn(move || {
                Command::new(path)
                    .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
                    .current_dir(dir)
                    .args(["-addon", name.as_str()])
                    .spawn()
                    .expect("failed to start executable")
                    .wait()
                    .expect("failed to actually run executable");
            });
        });

    app.run()
}

fn working_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::new());
    path.pop();
    path
}

fn canonicalize(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    let canon = path.canonicalize().unwrap_or(path);
    let canon = canon.display().to_string();
    canon.trim_start_matches(r"\\?\").to_string()
}

pub(crate) fn pick_path(
    app: slint::Weak<App>,
    title: &'static str,
    fun: impl FnMut(App, SharedString) + 'static + Clone,
) -> impl FnMut(SharedString) {
    move |current| {
        let app = app.unwrap();
        let mut fun = fun.clone();
        slint::spawn_local(async move {
            let current = current.as_str();
            let dlg = rfd::AsyncFileDialog::new()
                .set_parent(&app.window().window_handle())
                .set_can_create_directories(false)
                .set_title(title);
            let dlg = if current.is_empty() {
                dlg.set_directory(env::home_dir().unwrap_or_default())
            } else {
                dlg.set_directory(current)
            };
            let path = dlg.pick_folder().await;
            let Some(path) = path else {
                return;
            };
            let path = path.path().to_string_lossy().to_string().into();
            fun(app, path);
        })
        .expect("Slint event loop should already be initialized");
    }
}
