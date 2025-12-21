#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::env;
use std::path::PathBuf;

mod config;
mod projects;
mod setup;
use config::Config;
use slint::SharedString;


fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    Config::ensure_exists();

    let app = App::new()?;

    config::plugin(&app);
    setup::plugin(&app);
    projects::plugin(&app);

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
