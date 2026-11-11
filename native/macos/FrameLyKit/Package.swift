// swift-tools-version:5.9
import PackageDescription

// FrameLyKit: FrameLy's macOS media engine (ScreenCaptureKit, AVFoundation).
// Built as a static library and linked into the Rust app by src-tauri/build.rs.
let package = Package(
    name: "FrameLyKit",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "FrameLyKit", type: .static, targets: ["FrameLyKit"]),
    ],
    targets: [
        .target(
            name: "FrameLyKit",
            swiftSettings: [.unsafeFlags(["-warnings-as-errors"])]
        ),
        .testTarget(name: "FrameLyKitTests", dependencies: ["FrameLyKit"]),
    ]
)
