//! API keys in the Windows Credential Manager (generic credentials, local machine, current user).

use crate::core::errors::{AppError, AppResult};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::ERROR_NOT_FOUND,
        Security::Credentials::{
            CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
        },
    },
};

fn target(account: &str) -> HSTRING {
    HSTRING::from(format!("FrameLy/ai/{account}"))
}

fn is_not_found(e: &windows::core::Error) -> bool {
    e.code() == ERROR_NOT_FOUND.to_hresult()
}

pub fn set(account: &str, secret: &str) -> AppResult<()> {
    let target = target(account);
    let user = HSTRING::from("FrameLy");
    let blob = secret.as_bytes();
    let cred = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target.as_ptr() as *mut u16),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(user.as_ptr() as *mut u16),
        ..Default::default()
    };
    // SAFETY: every pointer in `cred` refers to data alive for the duration of the call;
    // CredWriteW copies it.
    unsafe { CredWriteW(&cred, 0) }.map_err(|e| AppError::Platform(format!("Credential Manager: {e}")))
}

pub fn get(account: &str) -> AppResult<Option<String>> {
    let target = target(account);
    let mut cred: *mut CREDENTIALW = std::ptr::null_mut();
    // SAFETY: valid target string and out-pointer; the returned block is freed with CredFree.
    unsafe {
        match CredReadW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, None, &mut cred) {
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(AppError::Platform(format!("Credential Manager: {e}"))),
            Ok(()) => {
                let c = &*cred;
                let bytes = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize);
                let secret = String::from_utf8_lossy(bytes).into_owned();
                CredFree(cred as *const _);
                Ok(Some(secret))
            }
        }
    }
}

pub fn delete(account: &str) -> AppResult<()> {
    let target = target(account);
    // SAFETY: valid target string.
    match unsafe { CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, None) } {
        Err(e) if !is_not_found(&e) => Err(AppError::Platform(format!("Credential Manager: {e}"))),
        _ => Ok(()),
    }
}
