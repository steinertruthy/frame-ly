use crate::platform::PermissionState;
use windows::{
    core::{w, PCWSTR},
    Graphics::Capture::GraphicsCaptureSession,
    Win32::System::Registry::{RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ},
};

/// Desktop apps can always use Windows.Graphics.Capture when the OS supports it.
pub fn screen() -> PermissionState {
    match GraphicsCaptureSession::IsSupported() {
        Ok(true) => PermissionState::Granted,
        _ => PermissionState::Restricted,
    }
}

fn read_value(root: HKEY, subkey: PCWSTR) -> Option<String> {
    let mut buf = [0u16; 32];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer and byte size are consistent; value is a short REG_SZ ("Allow"/"Deny").
    let status = unsafe {
        RegGetValueW(
            root,
            subkey,
            w!("Value"),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.is_err() {
        return None;
    }
    let chars = (size as usize / 2).saturating_sub(1);
    Some(String::from_utf16_lossy(&buf[..chars]))
}

/// Windows privacy switches for the microphone. Desktop (unpackaged) apps are governed by the
/// global switch (HKLM, set by policy/admin) plus the user switches for all apps and for
/// "desktop apps" (`NonPackaged`). Any "Deny" blocks capture.
pub fn microphone() -> PermissionState {
    const KEY: PCWSTR =
        w!(r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone");
    const DESKTOP: PCWSTR = w!(
        r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone\NonPackaged"
    );
    let values = [
        read_value(HKEY_LOCAL_MACHINE, KEY),
        read_value(HKEY_CURRENT_USER, KEY),
        read_value(HKEY_CURRENT_USER, DESKTOP),
    ];
    if values.iter().flatten().any(|v| v.eq_ignore_ascii_case("Deny")) {
        PermissionState::Denied
    } else {
        PermissionState::Granted
    }
}
