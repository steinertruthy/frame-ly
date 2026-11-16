fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    // macOS: compile the Swift media engine (native/macos/FrameLyKit) as a static library and
    // link it together with the Swift runtime and system frameworks it uses.
    if os == "macos" {
        link_swift_engine();
        for fw in ["ScreenCaptureKit", "AVFoundation", "CoreMedia", "CoreGraphics", "AppKit", "Security"] {
            println!("cargo:rustc-link-lib=framework={fw}");
        }
        // Apple Intelligence only exists on macOS 26+: weak-link so FrameLy still opens on 14/15.
        println!("cargo:rustc-link-arg=-Wl,-weak_framework,FoundationModels");
        // The Swift runtime ships with macOS 14+ in the dyld cache under /usr/lib/swift.
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        println!("cargo:rerun-if-changed=../native/macos/FrameLyKit/Sources");
        println!("cargo:rerun-if-changed=../native/macos/FrameLyKit/Package.swift");
    }
    if os == "windows" {
        // whisper.cpp's CPU backend reads the registry; whisper-rs-sys only adds this when the
        // build *host* is Windows.
        println!("cargo:rustc-link-lib=advapi32");
    }
    ensure_llm_worker();
    tauri_build::build()
}

/// The Swift engine can only be built on a macOS host (swift-rs is a macOS-only build dependency).
#[cfg(target_os = "macos")]
fn link_swift_engine() {
    swift_rs::SwiftLinker::new("14.0").with_package("FrameLyKit", "../native/macos/FrameLyKit/").link();
}
#[cfg(not(target_os = "macos"))]
fn link_swift_engine() {
    panic!("the macOS app must be built on macOS (Swift engine)");
}

/// The summary/topics runtime (`framely-llm`, built from llm-worker/ by
/// scripts/build-llm-worker.sh) is bundled as a Tauri sidecar, which must exist at build time.
fn ensure_llm_worker() {
    let target = std::env::var("TARGET").unwrap_or_default();
    let ext = if target.contains("windows") { ".exe" } else { "" };
    let path = std::path::PathBuf::from(format!("binaries/framely-llm-{target}{ext}"));
    println!("cargo:rerun-if-changed={}", path.display());
    if path.exists() {
        return;
    }
    // The Windows GNU target is only used to lint on macOS; it never produces an app.
    if target.ends_with("windows-gnu") {
        std::fs::create_dir_all("binaries").expect("binaries dir");
        std::fs::write(&path, b"lint-only placeholder").expect("placeholder");
        return;
    }
    panic!(
        "missing {}: build the local AI runtime first with `scripts/build-llm-worker.sh {target}` \
         (pnpm tauri dev/build run it automatically)",
        path.display()
    );
}
