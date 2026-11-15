//! Secure storage for API keys of optional external AI providers and the FrameLy AI Cloud device
//! key: the macOS Keychain or the Windows Credential Manager. Secrets never touch settings,
//! databases or logs.
//!
//! Reads are cached for the session and presence checks never read the secret, so macOS asks
//! for the keychain password at most once (and only for an item another build created).

use crate::core::errors::AppResult;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

pub fn set(account: &str, secret: &str) -> AppResult<()> {
    os::set(account, secret)?;
    cache().lock().unwrap().insert(account.into(), Some(secret.into()));
    Ok(())
}

pub fn get(account: &str) -> AppResult<Option<String>> {
    if let Some(v) = cache().lock().unwrap().get(account) {
        return Ok(v.clone());
    }
    let v = os::get(account)?;
    cache().lock().unwrap().insert(account.into(), v.clone());
    Ok(v)
}

/// Whether a secret is stored, without reading it (no keychain prompt).
pub fn exists(account: &str) -> AppResult<bool> {
    if let Some(v) = cache().lock().unwrap().get(account) {
        return Ok(v.is_some());
    }
    os::exists(account)
}

pub fn delete(account: &str) -> AppResult<()> {
    os::delete(account)?;
    cache().lock().unwrap().insert(account.into(), None);
    Ok(())
}

#[cfg(target_os = "macos")]
mod os {
    pub use crate::platform::macos::{
        keychain_delete as delete, keychain_exists as exists, keychain_get as get, keychain_set as set,
    };
}

#[cfg(windows)]
mod os {
    pub use crate::platform::windows::keystore::{delete, get, set};
    pub fn exists(account: &str) -> crate::core::errors::AppResult<bool> {
        Ok(get(account)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    /// Real round trip through the OS store (writes and removes a throwaway entry).
    #[test]
    #[ignore = "touches the login keychain"]
    fn round_trip() {
        let account = "framely-test-account";
        super::set(account, "sk-test-1").unwrap();
        super::set(account, "sk-test-2").unwrap();
        assert_eq!(super::get(account).unwrap().as_deref(), Some("sk-test-2"));
        assert!(super::exists(account).unwrap());
        super::delete(account).unwrap();
        assert_eq!(super::get(account).unwrap(), None);
        assert!(!super::exists(account).unwrap());
        super::delete(account).unwrap();
    }
}
