#![expect(
    clippy::undocumented_unsafe_blocks,
    reason = "This is all raw WinAPI calls lol"
)]

use libloading::{Library, Symbol};
use std::ffi::CString;
use std::ptr::null_mut;
use std::sync::{Arc, LazyLock, Mutex};
use std::{env, iter, ptr};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, thread, time::Duration};
use winapi::shared::minwindef::{HINSTANCE, MAX_PATH};
use winapi::shared::ntdef::LPSTR;
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleA};
use winapi::um::stringapiset::WideCharToMultiByte;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, IsWindow, IsWindowVisible, SWP_FRAMECHANGED, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};
use windows_sys::Win32::{
    Foundation::{BOOL, LPARAM},
    UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, GetWindowTextLengthW, GetWindowTextW,
    },
};

type Source2MainFn = unsafe extern "C" fn(
    image_base: HINSTANCE, // Base address of exe (HMODULE)
    reserved: HINSTANCE,   // Always null
    cmdline: LPSTR,        // Command line string
    window_mode: i32,      // Window show mode (SW_HIDE = 0, SW_SHOWNORMAL = 1)
    dir: *const i8,        // Maybe directory containing the exe?
    app_name: *const i8,   // App identifier ("csgo")
) -> u64;

pub(crate) fn run() {
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
            command_line_arguments(&[
                // We want Source 2 Tools
                "-tools",
                // The "game" is the hollowed-out core of cs2
                // Todo: find out how to rename this so that everything still works
                "-game core",
                // Need to run an addon, any addon, to save stuff properly
                "-addon foo",
                // Min size before asserts trigger when run with -dev
                "-w 4",
                "-h 4",
                // Hide it away
                "-x -4",
                "-y -4",
                "-noborder",
                // Allow opening multiple instances
                "-allowmultiple",
            ]),
            0,
            sz_base_dir_utf8.as_mut_ptr(),
            CString::new("core").unwrap().as_ptr(),
        );
        if result != 0 {
            panic!("Error with code {result}");
        }
    }
}

fn command_line_arguments(extra: &[&str]) -> *mut i8 {
    let extra = extra.iter().map(OsString::from);
    let args: Vec<std::ffi::OsString> = env::args_os().collect();
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return null_mut();
    };
    let all_args = iter::once(first)
        .chain(extra)
        .chain(args)
        .collect::<Vec<_>>()
        .join(&OsString::from(" "));
    CString::new(all_args.to_string_lossy().as_ref())
        .unwrap()
        .into_raw()
}

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
            if title == "Core" {
                let Ok(window_state) = WINDOW_STATE.try_lock() else {
                    eprintln!("failed to lock window state");
                    return;
                };
                if *window_state == WindowState::Uninit {
                    hide_from_taskbar(hwnd);
                }
            }
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

unsafe fn hide_from_taskbar(hwnd: isize) {
    unsafe {
        let mut ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;
        ex |= WS_EX_TOOLWINDOW as isize; // mark as tool window
        ex &= !(WS_EX_APPWINDOW as isize); // remove app window flag
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex);
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );
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
        let Ok(window_state) = WINDOW_STATE.try_lock().map(|w| *w) else {
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
                *window_state = WindowState::Searching;
            }
        }
        EnumWindows(Some(enum_windows_proc), 0);
        let Ok(window_state) = WINDOW_STATE.try_lock() else {
            eprintln!("failed to lock window state");
            return true;
        };
        match *window_state {
            WindowState::Searching => true,
            WindowState::Open | WindowState::Uninit => false,
        }
    }
}
