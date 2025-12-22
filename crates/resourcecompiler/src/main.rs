#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    path::PathBuf,
    process::{Command, exit},
};

fn main() {
    let mut args = env::args_os();
    let _program_name = args.next();

    let status = Command::new(working_dir().join("resourcecompiler_inner.exe"))
        .args(args)
        .status()
        .expect("failed to execute process");

    exit(status.code().unwrap_or(1));
}

fn working_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::new());
    path.pop();
    path
}
