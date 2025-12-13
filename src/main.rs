#![windows_subsystem = "windows"]

use libloading::{Library, Symbol};
use std::ffi::CString;
use std::ptr;
use std::ptr::null_mut;
use std::sync::{Arc, LazyLock, Mutex};
use winapi::shared::minwindef::{HINSTANCE, MAX_PATH};
use winapi::shared::ntdef::LPSTR;
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleA};
use winapi::um::processenv::GetCommandLineA;
use winapi::um::stringapiset::WideCharToMultiByte;
use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, IsWindowVisible};

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

        thread::spawn(|| {
            loop {
                if asset_browser_closed() {
                    std::process::exit(0);
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        let result = source2_main(
            GetModuleHandleA(ptr::null()),
            HINSTANCE::default(),
            GetCommandLineA(),
            0,
            sz_base_dir_utf8.as_mut_ptr(),
            CString::new("core").unwrap().as_ptr(),
        );
        if result != 0 {
            panic!("Error with code {result}");
        }
    }
}

use std::{ffi::OsString, os::windows::ffi::OsStringExt, thread, time::Duration};
use windows_sys::Win32::{
    Foundation::{BOOL, LPARAM},
    UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, GetWindowTextLengthW, GetWindowTextW,
    },
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum WindowState {
    Uninit,
    Searching,
    Open,
}

static WINDOW_STATE: LazyLock<Arc<Mutex<WindowState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(WindowState::Uninit)));

unsafe fn check_window_text(hwnd: isize) {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            if IsWindow(hwnd) == 0 || IsWindowVisible(hwnd) == 0 {
                return;
            }

            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return;
            }

            let mut buf = vec![0u16; (len + 1) as usize];
            GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);

            let title = OsString::from_wide(&buf[..len as usize])
                .to_string_lossy()
                .into_owned();
            if title == "Asset Browser" {
                let Ok(mut window_state) = WINDOW_STATE.try_lock() else {
                    eprintln!("failed to lock window state");
                    return;
                };
                *window_state = WindowState::Open;
            }
        }
    }
}

unsafe extern "system" fn enum_child_proc(hwnd: isize, _: LPARAM) -> BOOL {
    unsafe {
        check_window_text(hwnd);
        let Ok(window_state) = WINDOW_STATE.try_lock() else {
            eprintln!("failed to lock window state");
            return 0;
        };
        if *window_state == WindowState::Open {
            0
        } else {
            1
        }
    }
}

unsafe fn enum_all_children(hwnd: isize) {
    unsafe {
        EnumChildWindows(hwnd, Some(enum_child_proc), 0);
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: isize, _: LPARAM) -> BOOL {
    unsafe {
        check_window_text(hwnd);
        let Ok(window_state) = WINDOW_STATE.try_lock().map(|w| w.clone()) else {
            eprintln!("failed to lock window state");
            return 0;
        };
        if window_state != WindowState::Open {
            enum_all_children(hwnd);
        }
        let Ok(window_state) = WINDOW_STATE.try_lock() else {
            eprintln!("failed to lock window state");
            return 0;
        };
        if *window_state == WindowState::Open {
            0
        } else {
            1
        }
    }
}

fn asset_browser_closed() -> bool {
    unsafe {
        {
            let Ok(mut window_state) = WINDOW_STATE.try_lock() else {
                eprintln!("failed to lock window state");
                return true;
            };
            if *window_state == WindowState::Open {
                *window_state = WindowState::Searching
            }
        }
        EnumWindows(Some(enum_windows_proc), 0);
        let Ok(window_state) = WINDOW_STATE.try_lock() else {
            eprintln!("failed to lock window state");
            return true;
        };
        match *window_state {
            WindowState::Uninit => false,
            WindowState::Searching => true,
            WindowState::Open => false,
        }
    }
}
