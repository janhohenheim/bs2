#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod windows;

fn main() {
    #[cfg(target_os = "windows")]
    windows::run();
    #[cfg(not(target_os = "windows"))]
    panic!("This launcher is only supported on Windows");
}
