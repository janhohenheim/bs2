use std::{fs, iter, os::windows::process::CommandExt as _, path::PathBuf, process::Command};

use slint::{ComponentHandle as _, Model as _, ModelRc};

use crate::{
    App, Bs2Config, ProjectDef, ProjectsPageLogic,
    config::{Config, Project},
    pick_path, working_dir,
};

pub(super) fn plugin(app: &App) {
    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_update_project_selection(move || {
            let app = app_inner.unwrap();
            let new_projects = iter::once("➕︎ Create New Project")
                .chain(Config::read().projects.iter().map(|p| p.name.as_str()))
                .map(std::convert::Into::into)
                .collect::<Vec<_>>();
            app.global::<ProjectsPageLogic>()
                .set_project_selection(ModelRc::from(new_projects.as_ref()));
            app.global::<Bs2Config>()
                .set_last_project((new_projects.len() - 1) as i32);
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
            app.global::<ProjectsPageLogic>()
                .set_new_project_name("".into());
            app.global::<ProjectsPageLogic>()
                .set_new_project_path("".into());
            config.write();
        });
    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_launch_tools(move |name| {
            let app = app_inner.unwrap();
            let index = app
                .global::<ProjectsPageLogic>()
                .get_project_selection()
                .iter()
                .position(|p| p == name)
                .unwrap_or_default();
            app.global::<Bs2Config>().set_last_project(index as i32);
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

    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_validate_new_project_name(move |name| {
            let app = app_inner.unwrap();
            if name.is_empty() {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_name_err("".into());
            } else if is_ascii_rust_ident(name.as_str()) {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_name_err("".into());
            } else {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_name_err("Project name must start with a letter, only contain English letters, digits, or the separator _, and have no spaces.".into());
            }
        });

    let app_inner = app.as_weak();
    app.global::<ProjectsPageLogic>()
        .on_validate_new_project_path(move |path| {
            let app = app_inner.unwrap();
            if path.is_empty() {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_path_err("".into());
            } else if !PathBuf::from(path.as_str()).exists() {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_path_err("Project path doesn't exist".into());
            } else {
                app.global::<ProjectsPageLogic>()
                    .set_new_project_path_err("".into());
            }
        });
}

fn is_ascii_rust_ident(s: &str) -> bool {
    let mut chars = s.bytes();

    let Some(first) = chars.next() else {
        return false;
    };

    match first {
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {}
        _ => return false,
    }

    chars.all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'))
}
