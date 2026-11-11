fn main() {
    // macOS: compile the Swift media engine (native/macos/FrameLyKit) as a static library and
    // link it together with the Swift runtime and system frameworks it uses.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        swift_rs::SwiftLinker::new("14.0")
            .with_package("FrameLyKit", "../native/macos/FrameLyKit/")
            .link();
        for fw in [
            "ScreenCaptureKit",
            "AVFoundation",
            "CoreMedia",
            "CoreGraphics",
            "AppKit",
        ] {
            println!("cargo:rustc-link-lib=framework={fw}");
        }
        // The Swift runtime ships with macOS 14+ in the dyld cache under /usr/lib/swift.
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        println!("cargo:rerun-if-changed=../native/macos/FrameLyKit/Sources");
        println!("cargo:rerun-if-changed=../native/macos/FrameLyKit/Package.swift");
    }
    tauri_build::build()
}
