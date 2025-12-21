#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::iter;
use std::path::PathBuf;

mod config;
mod setup;
use config::Config;
use slint::Model;
use slint::ModelRc;

use crate::config::Project;
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
    app.global::<SetupPageLogic>()
        .on_pick_cs2(pick_cs2(app_inner));

    let app_inner = app.as_weak();
    app.global::<SetupPageLogic>().on_write_config(move || {
        let app = app_inner.unwrap();
        Config {
            cs2_path: app.global::<Bs2Config>().get_cs2_path().into(),
            ..Config::read()
        }
        .write();
    });

    app.global::<ProjectsPageLogic>()
        .on_project_selection(move || {
            ModelRc::from(
                iter::once("➕︎ Create New Project")
                    .chain(Config::read().projects.iter().map(|p| p.name.as_str()))
                    .map(|s| s.into())
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
        });
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

    app.run()
}

fn canonicalize(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    let canon = path.canonicalize().unwrap_or(path);
    let canon = canon.display().to_string();
    canon.trim_start_matches(r"\\?\").to_string()
}
