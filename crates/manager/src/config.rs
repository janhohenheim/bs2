use serde::{Deserialize, Serialize};
use slint::ComponentHandle as _;
use std::{fs, path::PathBuf, sync::LazyLock};
use tracing::{error, info};

use crate::{App, Bs2Config, canonicalize, working_dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) cs2_path: String,
    pub(crate) projects: Vec<Project>,
    pub(crate) last_project: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cs2_path:
                r"C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\"
                    .into(),
            projects: Vec::new(),
            last_project: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) path: String,
}

static PATH: LazyLock<PathBuf> = LazyLock::new(|| working_dir().join("config.toml"));
static CANON_PATH: LazyLock<String> = LazyLock::new(|| canonicalize(&*PATH));

impl Config {
    pub(crate) fn read() -> Self {
        match fs::read_to_string(PATH.as_path()) {
            Ok(contents) => toml::from_str(&contents)
                .map(|cfg| {
                    info!(
                        "Successfully read config from {} with contents {cfg:#?}",
                        *CANON_PATH
                    );
                    cfg
                })
                .unwrap_or_else(|e| {
                    error!("Config at {} contains invalid data: {e:?}", *CANON_PATH);
                    Self::default()
                }),
            Err(e) => {
                error!("Failed to read config from {}: {e:?}", *CANON_PATH);
                Self::default()
            }
        }
    }

    pub(crate) fn write(&self) {
        let toml_str = match toml::to_string_pretty(self) {
            Ok(toml) => toml,
            Err(e) => {
                error!("Serialize config with contents {self:#?}: {e:?}",);
                return;
            }
        };
        if let Err(e) = fs::write(PATH.as_path(), toml_str) {
            error!(
                "Failed to write config to {} with contents {self:#?}: {e:?}",
                *CANON_PATH
            );
        } else {
            info!(
                "Successfully wrote config to {} with contents {self:#?}",
                *CANON_PATH
            );
        }
    }

    pub(crate) fn ensure_exists() {
        match fs::exists(PATH.as_path()) {
            Ok(true) => {
                info!(
                    "No need to init config because it already exists at {}",
                    *CANON_PATH
                );
                return;
            }
            Ok(false) => {
                info!("Initializing default config at {}", *CANON_PATH);
            }
            Err(e) => {
                error!("Failed to check if config at {} exists: {e:?}", *CANON_PATH);
            }
        }
        let config = Self::default();
        config.write();
    }
}

pub(super) fn plugin(app: &App) {
    let config = Config::read();
    app.global::<Bs2Config>()
        .set_cs2_path(config.cs2_path.clone().into());
    app.global::<Bs2Config>()
        .set_last_project(config.last_project as i32);
    let app_inner = app.as_weak();
    app.global::<Bs2Config>().on_write_config(move || {
        let app = app_inner.unwrap();
        Config {
            cs2_path: app.global::<Bs2Config>().get_cs2_path().into(),
            last_project: app.global::<Bs2Config>().get_last_project() as u32,
            ..Config::read()
        }
        .write();
    });
}
