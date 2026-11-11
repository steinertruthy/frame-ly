use crate::{
    core::errors::AppResult,
    platform::{Rect, WindowInfo},
};
use std::{mem::size_of, path::Path};
use windows::{
    core::{BOOL, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, RECT},
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS},
        System::Threading::{
            GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindow, GetWindowLongPtrW, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, GWL_EXSTYLE,
            GW_OWNER, WS_EX_TOOLWINDOW,
        },
    },
};

/// Shell surfaces that are never "an application window".
const EXCLUDED_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Windows.UI.Core.CoreWindow",
];

unsafe extern "system" fn collect(hwnd: HWND, data: LPARAM) -> BOOL {
    // SAFETY: `data` is the &mut Vec passed by `list` for the duration of EnumWindows.
    let out = unsafe { &mut *(data.0 as *mut Vec<HWND>) };
    out.push(hwnd);
    true.into()
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    // SAFETY: valid buffer; returns the number of chars written.
    let n = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn title(hwnd: HWND) -> String {
    // SAFETY: buffer sized from GetWindowTextLengthW (+1 for NUL).
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; len as usize + 1];
        let n = GetWindowTextW(hwnd, &mut buf) as usize;
        String::from_utf16_lossy(&buf[..n])
    }
}

/// Alt-Tab–style filter: visible, not minimized, unowned, not a tool window, not cloaked
/// (UWP apps suspended or on another virtual desktop report as visible but cloaked).
fn is_capturable(hwnd: HWND) -> bool {
    // SAFETY: read-only queries on a window handle from EnumWindows.
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
            return false;
        }
        if GetWindow(hwnd, GW_OWNER)
            .map(|h| !h.is_invalid())
            .unwrap_or(false)
        {
            return false;
        }
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
            return false;
        }
        let mut cloaked = 0u32;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            size_of::<u32>() as u32,
        )
        .is_ok()
            && cloaked != 0
        {
            return false;
        }
    }
    !EXCLUDED_CLASSES.contains(&class_name(hwnd).as_str())
}

/// Visible frame bounds (without the invisible resize borders of Windows 10/11).
fn bounds(hwnd: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    // SAFETY: out-pointer to a RECT of the size we pass.
    unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut r as *mut _ as *mut _,
            size_of::<RECT>() as u32,
        )
    }
    .ok()?;
    Some(Rect {
        x: r.left as f64,
        y: r.top as f64,
        width: (r.right - r.left) as f64,
        height: (r.bottom - r.top) as f64,
    })
}

fn exe_path(pid: u32) -> Option<String> {
    // SAFETY: handle is closed before returning; buffer and size are consistent.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

pub fn list() -> AppResult<Vec<WindowInfo>> {
    let mut handles: Vec<HWND> = Vec::new();
    // SAFETY: the callback only pushes into `handles`, which outlives the call.
    unsafe { EnumWindows(Some(collect), LPARAM(&mut handles as *mut _ as isize)) }
        .map_err(|e| super::win_err("EnumWindows", e))?;

    // SAFETY: trivial query.
    let own_pid = unsafe { GetCurrentProcessId() };
    let mut out = Vec::new();
    for hwnd in handles {
        if !is_capturable(hwnd) {
            continue;
        }
        let mut pid = 0u32;
        // SAFETY: valid handle and out-pointer.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == own_pid {
            continue;
        }
        let title = title(hwnd);
        let Some(bounds) = bounds(hwnd).filter(|b| b.width >= 40.0 && b.height >= 40.0) else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        let exe = exe_path(pid);
        let app_name = exe
            .as_deref()
            .and_then(|p| Path::new(p).file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| title.clone());
        out.push(WindowInfo {
            id: format!("{:#x}", hwnd.0 as usize),
            title,
            app_name,
            app_id: exe,
            pid,
            bounds,
        });
    }
    Ok(out)
}
