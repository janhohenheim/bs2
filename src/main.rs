use libloading::{Library, Symbol};
use std::ffi::CString;
use std::ptr;
use std::ptr::null_mut;
use winapi::shared::minwindef::{HINSTANCE, MAX_PATH};
use winapi::shared::ntdef::LPSTR;
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleA};
use winapi::um::processenv::GetCommandLineA;
use winapi::um::stringapiset::WideCharToMultiByte;

//mod codegen;

type Source2MainFn = unsafe extern "C" fn(
    image_base: HINSTANCE, // Base address of exe (HMODULE)
    reserved: HINSTANCE,   // Always null
    cmdline: LPSTR,        // Command line string
    window_mode: i32,      // Window show mode (SW_HIDE = 0, SW_SHOWNORMAL = 1)
    dir: *const i8,        // Maybe directory containing the exe?
    app_name: *const i8,   // App identifier ("csgo")
) -> u64;

fn main() {
    unsafe {
        // Load engine2.dll - must be in same dir or on PATH
        let lib = Library::new("engine2.dll").expect("Failed to load engine2.dll");

        let source2_main: Symbol<Source2MainFn> = lib
            .get(b"Source2Main\0")
            .expect("Failed to find Source2Main");

        let mut sz_base_dir: [u16; MAX_PATH] = [0; MAX_PATH];
        let mut sz_base_dir_utf8: [i8; MAX_PATH] = [0; MAX_PATH];

        if GetModuleFileNameW(null_mut(), sz_base_dir.as_mut_ptr(), MAX_PATH as u32) == 0 {
            panic!("GetModuleFileName failed");
        }
        let len = sz_base_dir.iter().position(|&c| c == 0).unwrap_or(MAX_PATH);
        let last_backslash = sz_base_dir[..len]
            .iter()
            .rposition(|&c| c == '\\' as u16)
            .unwrap_or(0);
        sz_base_dir[last_backslash] = 0;

        // Convert base path to UTF-8
        if WideCharToMultiByte(
            65001, // CP_UTF8
            0,
            sz_base_dir.as_ptr(),
            -1,
            sz_base_dir_utf8.as_mut_ptr(),
            MAX_PATH as i32,
            null_mut(),
            null_mut(),
        ) == 0
        {
            panic!("Could not convert base path");
        }

        let result = source2_main(
            GetModuleHandleA(ptr::null()),
            HINSTANCE::default(),
            GetCommandLineA(),
            0,
            sz_base_dir_utf8.as_mut_ptr(),
            CString::new("core").unwrap().as_ptr(),
        );

        println!("  -> returned: {}", result);
    }
}
