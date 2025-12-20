#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    cs2_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { cs2_path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive\\".into() }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();

    let app = App::new()?;
    let app_inner = app.as_weak();
    app.set_setup_done(is_setup_done());
    {
        let setup_page = app.global::<SetupPageLogic>();
        setup_page.on_run_setup(|value| run_setup(&value));
        setup_page.on_pick_cs2(pick_cs2(app_inner));
    }

    app.run()
}

fn is_setup_done() -> bool {
    false
}

fn run_setup(path: &str) {
    info!("Setup: {path}");
}

fn pick_cs2(app: slint::Weak<App>) -> impl FnMut() {
    move || {
        let app = app.unwrap();
        slint::spawn_local(async move {
            let current = app.global::<SetupPageLogic>().get_cs2_path();
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
            app.global::<SetupPageLogic>()
                .set_cs2_path(path.path().to_string_lossy().to_string().into());
        })
        .expect("Slint event loop should already be initialized");
    }
}

fn read_config() {}
