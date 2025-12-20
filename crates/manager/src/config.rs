use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) cs2_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { cs2_path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive\\".into() }
    }
}

static PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("config.toml"));
static CANON_PATH: LazyLock<String> = LazyLock::new(|| {
    let canon = PATH.canonicalize().unwrap_or(PATH.to_path_buf());
    let disp = canon.display().to_string();
    disp.trim_start_matches("\\\\?\\").to_string()
});

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
                error!("Serialize config with contents {self:?}: {e:?}",);
                return;
            }
        };
        if let Err(e) = fs::write(PATH.as_path(), toml_str) {
            error!(
                "Failed to write config to {} with contents {self:?}: {e:?}",
                *CANON_PATH
            )
        } else {
            info!(
                "Successfully wrote config to {} with contents {self:?}",
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
                error!("Failed to check if config at {} exists: {e:?}", *CANON_PATH)
            }
        }
        let config = Self::default();
        config.write();
    }
}
