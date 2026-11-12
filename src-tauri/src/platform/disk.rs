//! Free space on the volume holding a path.

use std::path::Path;

#[cfg(unix)]
pub fn free_bytes(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: valid NUL-terminated path and an out-struct of the right type.
    (unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0).then(|| st.f_bavail as u64 * st.f_frsize as u64)
}

#[cfg(windows)]
pub fn free_bytes(path: &Path) -> Option<u64> {
    use windows::{core::HSTRING, Win32::Storage::FileSystem::GetDiskFreeSpaceExW};
    let mut free = 0u64;
    // SAFETY: valid wide string and out-pointer.
    unsafe { GetDiskFreeSpaceExW(&HSTRING::from(path.as_os_str()), Some(&mut free), None, None) }.ok()?;
    Some(free)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_free_space_for_temp_dir() {
        let free = super::free_bytes(&std::env::temp_dir()).unwrap();
        assert!(free > 0);
    }
}
