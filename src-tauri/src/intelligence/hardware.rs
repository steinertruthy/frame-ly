//! What this computer can run: memory, processor, graphics and acceleration. Drives the
//! recommended FrameLy AI profile and the time estimates. Never assumes a GPU accelerates: only
//! Apple Silicon (Metal) is used; everything else runs on the CPU.

use crate::core::settings::AiProfile;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub os: &'static str,
    pub cpu: String,
    pub cores: usize,
    pub ram_bytes: u64,
    pub gpu: Option<String>,
    pub vram_bytes: Option<u64>,
    /// "metal" | "cpu"
    pub accel: &'static str,
    /// x86 only: whisper.cpp/llama.cpp are built for AVX2 there.
    pub avx2: bool,
    /// Local AI can run here at all.
    pub supported: bool,
    pub recommended: AiProfile,
}

const GB: u64 = 1024 * 1024 * 1024;

/// The profile FrameLy suggests. Pure so it can be tested against example machines.
pub fn recommend(ram_bytes: u64, accel: &str) -> AiProfile {
    let ram_gb = ram_bytes as f64 / GB as f64;
    if accel == "metal" {
        if ram_gb >= 23.5 {
            AiProfile::Advanced
        } else if ram_gb >= 11.5 {
            AiProfile::Balanced
        } else {
            AiProfile::Basic
        }
    } else if ram_gb >= 15.5 {
        // CPU only: the Advanced models are too slow to suggest by default.
        AiProfile::Balanced
    } else {
        AiProfile::Basic
    }
}

/// The profile actually used: the user's choice, or the recommendation.
pub fn resolve(choice: AiProfile, hw: &Hardware) -> AiProfile {
    match choice {
        AiProfile::Auto => hw.recommended,
        p => p,
    }
}

pub fn detect() -> Hardware {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let (cpu, ram_bytes, gpu, vram_bytes) = os::probe();
    #[cfg(target_arch = "x86_64")]
    let avx2 = std::arch::is_x86_feature_detected!("avx2");
    #[cfg(not(target_arch = "x86_64"))]
    let avx2 = false;
    let accel = if cfg!(all(target_os = "macos", target_arch = "aarch64")) { "metal" } else { "cpu" };
    let supported = cfg!(target_arch = "aarch64") || avx2;
    Hardware {
        os: std::env::consts::OS,
        cpu,
        cores,
        ram_bytes,
        gpu,
        vram_bytes,
        accel,
        avx2,
        supported,
        recommended: recommend(ram_bytes, accel),
    }
}

#[cfg(target_os = "macos")]
mod os {
    fn sysctl_u64(name: &str) -> Option<u64> {
        let c = std::ffi::CString::new(name).ok()?;
        let mut v: u64 = 0;
        let mut len = std::mem::size_of::<u64>();
        // SAFETY: valid name and an out-buffer of the declared size.
        let r = unsafe {
            libc::sysctlbyname(c.as_ptr(), &mut v as *mut u64 as *mut _, &mut len, std::ptr::null_mut(), 0)
        };
        (r == 0).then_some(v)
    }

    fn sysctl_string(name: &str) -> Option<String> {
        let c = std::ffi::CString::new(name).ok()?;
        let mut len = 0usize;
        // SAFETY: size query, then a buffer of that size.
        unsafe {
            if libc::sysctlbyname(c.as_ptr(), std::ptr::null_mut(), &mut len, std::ptr::null_mut(), 0) != 0 {
                return None;
            }
            let mut buf = vec![0u8; len];
            if libc::sysctlbyname(c.as_ptr(), buf.as_mut_ptr() as *mut _, &mut len, std::ptr::null_mut(), 0)
                != 0
            {
                return None;
            }
            Some(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string())
        }
    }

    pub fn probe() -> (String, u64, Option<String>, Option<u64>) {
        let cpu = sysctl_string("machdep.cpu.brand_string").unwrap_or_else(|| "Mac".into());
        let ram = sysctl_u64("hw.memsize").unwrap_or(0);
        // Apple Silicon GPUs share the system memory.
        let gpu = cfg!(target_arch = "aarch64").then(|| format!("{cpu} GPU"));
        (cpu, ram, gpu, None)
    }
}

#[cfg(windows)]
mod os {
    use windows::Win32::{
        Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE},
        System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
    };

    fn cpu_name() -> String {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::__cpuid;
            // SAFETY: CPUID is always available on x86_64; leaves 0x80000002..4 hold the brand.
            // (`__cpuid` became safe in newer Rust; the block is for the 1.80 MSRV.)
            #[allow(unused_unsafe)]
            unsafe {
                if __cpuid(0x8000_0000).eax >= 0x8000_0004 {
                    let mut bytes = Vec::with_capacity(48);
                    for leaf in 0x8000_0002u32..=0x8000_0004 {
                        let r = __cpuid(leaf);
                        for v in [r.eax, r.ebx, r.ecx, r.edx] {
                            bytes.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                    return String::from_utf8_lossy(&bytes).trim_matches(char::from(0)).trim().to_string();
                }
            }
        }
        "PC".into()
    }

    pub fn probe() -> (String, u64, Option<String>, Option<u64>) {
        let mut mem =
            MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
        // SAFETY: struct with its size set, as the API requires.
        let ram = unsafe { GlobalMemoryStatusEx(&mut mem) }.map(|_| mem.ullTotalPhys).unwrap_or(0);
        // The largest hardware adapter (informational: models run on the CPU on Windows).
        let mut best: Option<(String, u64)> = None;
        // SAFETY: plain DXGI enumeration; descriptors are copied out.
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                let mut i = 0;
                while let Ok(adapter) = factory.EnumAdapters1(i) {
                    i += 1;
                    let Ok(desc) = adapter.GetDesc1() else { continue };
                    if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                        continue;
                    }
                    let name =
                        String::from_utf16_lossy(&desc.Description).trim_matches(char::from(0)).to_string();
                    let vram = desc.DedicatedVideoMemory as u64;
                    if best.as_ref().map_or(true, |(_, v)| vram > *v) {
                        best = Some((name, vram));
                    }
                }
            }
        }
        (cpu_name(), ram, best.as_ref().map(|b| b.0.clone()), best.map(|b| b.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendations_follow_memory_and_acceleration() {
        assert_eq!(recommend(8 * GB, "metal"), AiProfile::Basic);
        assert_eq!(recommend(16 * GB, "metal"), AiProfile::Balanced);
        assert_eq!(recommend(18 * GB, "metal"), AiProfile::Balanced);
        assert_eq!(recommend(32 * GB, "metal"), AiProfile::Advanced);
        assert_eq!(recommend(8 * GB, "cpu"), AiProfile::Basic);
        assert_eq!(recommend(16 * GB, "cpu"), AiProfile::Balanced);
        assert_eq!(recommend(64 * GB, "cpu"), AiProfile::Balanced, "no Advanced without acceleration");
    }

    #[test]
    fn detects_this_machine() {
        let hw = detect();
        assert!(hw.ram_bytes > GB && hw.cores > 0 && !hw.cpu.is_empty());
        assert!(hw.supported);
        assert_eq!(resolve(AiProfile::Basic, &hw), AiProfile::Basic);
        assert_eq!(resolve(AiProfile::Auto, &hw), hw.recommended);
    }
}
