use super::win_err;
use crate::{core::errors::AppResult, platform::AudioDevice};
use windows::{
    core::PWSTR,
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Media::Audio::{
            eCapture, eConsole, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
        },
    },
};

/// Reads and frees a COM-allocated wide string.
unsafe fn take_pwstr(p: PWSTR) -> String {
    // SAFETY: caller passes a valid CoTaskMem-allocated, NUL-terminated string.
    unsafe {
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

unsafe fn device_id(d: &IMMDevice) -> AppResult<String> {
    // SAFETY: GetId returns a CoTaskMem string we own.
    unsafe { Ok(take_pwstr(d.GetId().map_err(|e| win_err("IMMDevice::GetId", e))?)) }
}

unsafe fn friendly_name(d: &IMMDevice) -> Option<String> {
    // SAFETY: property store opened read-only; PROPVARIANT owned and dropped here.
    unsafe {
        let store = d.OpenPropertyStore(STGM_READ).ok()?;
        let value = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
        let s = value.to_string();
        (!s.is_empty()).then_some(s)
    }
}

pub fn list_inputs() -> AppResult<Vec<AudioDevice>> {
    // SAFETY: COM initialized for this (blocking worker) thread; ignore "already initialized"
    // with a different model, since Core Audio enumeration works in either apartment.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| win_err("MMDeviceEnumerator", e))?;
        let default_id =
            enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).ok().and_then(|d| device_id(&d).ok());
        let collection = enumerator
            .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            .map_err(|e| win_err("EnumAudioEndpoints", e))?;
        let count = collection.GetCount().map_err(|e| win_err("GetCount", e))?;
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let Ok(device) = collection.Item(i) else {
                continue;
            };
            let id = device_id(&device)?;
            out.push(AudioDevice {
                name: friendly_name(&device).unwrap_or_else(|| format!("Microphone {}", i + 1)),
                is_default: default_id.as_deref() == Some(id.as_str()),
                id,
            });
        }
        Ok(out)
    }
}
