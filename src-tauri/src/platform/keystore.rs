//! Secure storage for API keys of optional external AI providers: the macOS Keychain or the
//! Windows Credential Manager. Keys never touch settings, databases or logs.

#[cfg(target_os = "macos")]
use crate::core::errors::AppResult;

#[cfg(target_os = "macos")]
pub fn set(account: &str, secret: &str) -> AppResult<()> {
    super::macos::keychain_set(account, secret)
}

#[cfg(target_os = "macos")]
pub fn get(account: &str) -> AppResult<Option<String>> {
    super::macos::keychain_get(account)
}

#[cfg(target_os = "macos")]
pub fn delete(account: &str) -> AppResult<()> {
    super::macos::keychain_delete(account)
}

#[cfg(windows)]
pub use super::windows::keystore::{delete, get, set};

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
        super::delete(account).unwrap();
        assert_eq!(super::get(account).unwrap(), None);
        super::delete(account).unwrap();
    }
}
