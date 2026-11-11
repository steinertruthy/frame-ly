use super::win_err;
use crate::{
    core::errors::AppResult,
    platform::{Display, Rect},
};
use std::{collections::HashMap, mem::size_of};
use windows::{
    core::BOOL,
    Win32::{
        Devices::Display::{
            DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
        },
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW, DEVMODEW,
            ENUM_CURRENT_SETTINGS, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
        },
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            WindowsAndMessaging::MONITORINFOF_PRIMARY,
        },
    },
};

fn wide_to_string(w: &[u16]) -> String {
    let end = w.iter().position(|&c| c == 0).unwrap_or(w.len());
    String::from_utf16_lossy(&w[..end])
}

/// Maps GDI device names (`\\.\DISPLAY1`) to monitor friendly names ("DELL U2720Q").
fn friendly_names() -> HashMap<String, String> {
    let mut map = HashMap::new();
    // SAFETY: plain Win32 calls with correctly sized, owned buffers.
    unsafe {
        let (mut np, mut nm) = (0u32, 0u32);
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut np, &mut nm).is_err() {
            return map;
        }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); np as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); nm as usize];
        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut np,
            paths.as_mut_ptr(),
            &mut nm,
            modes.as_mut_ptr(),
            None,
        )
        .is_err()
        {
            return map;
        }
        for path in paths.iter().take(np as usize) {
            let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
            source.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            source.header.adapterId = path.sourceInfo.adapterId;
            source.header.id = path.sourceInfo.id;
            let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
            target.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target.header.size = size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            target.header.adapterId = path.targetInfo.adapterId;
            target.header.id = path.targetInfo.id;
            if DisplayConfigGetDeviceInfo(&mut source.header) == 0
                && DisplayConfigGetDeviceInfo(&mut target.header) == 0
            {
                let name = wide_to_string(&target.monitorFriendlyDeviceName);
                if !name.is_empty() {
                    map.insert(wide_to_string(&source.viewGdiDeviceName), name);
                }
            }
        }
    }
    map
}

unsafe extern "system" fn collect(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    // SAFETY: `data` is the &mut Vec passed by `list` for the duration of EnumDisplayMonitors.
    let out = unsafe { &mut *(data.0 as *mut Vec<HMONITOR>) };
    out.push(monitor);
    true.into()
}

pub fn list() -> AppResult<Vec<Display>> {
    let mut monitors: Vec<HMONITOR> = Vec::new();
    // SAFETY: the callback only pushes into `monitors`, which outlives the call.
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut monitors as *mut _ as isize),
        )
        .ok()
        .map_err(|e| win_err("EnumDisplayMonitors", e))?;
    }
    let names = friendly_names();

    let mut out = Vec::with_capacity(monitors.len());
    for (index, monitor) in monitors.into_iter().enumerate() {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        // SAFETY: MONITORINFOEXW starts with MONITORINFO and cbSize announces the extended size.
        let ok = unsafe {
            GetMonitorInfoW(
                monitor,
                &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
            )
        };
        if !ok.as_bool() {
            continue;
        }
        let device = wide_to_string(&info.szDevice);
        let r = info.monitorInfo.rcMonitor;
        let (width, height) = ((r.right - r.left) as u32, (r.bottom - r.top) as u32);

        let (mut dpi_x, mut dpi_y) = (96u32, 96u32);
        // SAFETY: valid monitor handle and out-pointers.
        let _ = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };

        let mut mode = DEVMODEW {
            dmSize: size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let wide_device: Vec<u16> = device.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: NUL-terminated device name and a DEVMODEW with dmSize set.
        let hz = unsafe {
            EnumDisplaySettingsW(
                windows::core::PCWSTR(wide_device.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut mode,
            )
        }
        .as_bool()
        .then_some(mode.dmDisplayFrequency)
        .filter(|&f| f > 1);

        out.push(Display {
            name: names
                .get(&device)
                .cloned()
                .unwrap_or_else(|| format!("Display {}", index + 1)),
            id: device,
            bounds: Rect {
                x: r.left as f64,
                y: r.top as f64,
                width: width as f64,
                height: height as f64,
            },
            pixel_width: width,
            pixel_height: height,
            scale_factor: dpi_x as f64 / 96.0,
            refresh_rate: hz.map(f64::from),
            is_primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    }
    Ok(out)
}
