import AVFoundation
import ImageIO
import UniformTypeIdentifiers

struct MediaInfo: Encodable {
    let width: Int?
    let height: Int?
    let durationMs: Int?
}

/// Duration and display size of a video file (rotation applied).
func videoInfo(_ path: String) throws -> MediaInfo {
    let asset = AVURLAsset(url: URL(fileURLWithPath: path))
    return try blocking {
        let duration = try await asset.load(.duration)
        var size: CGSize?
        if let track = try await asset.loadTracks(withMediaType: .video).first {
            let (natural, transform) = try await track.load(.naturalSize, .preferredTransform)
            let r = CGRect(origin: .zero, size: natural).applying(transform)
            size = CGSize(width: abs(r.width), height: abs(r.height))
        }
        return MediaInfo(
            width: size.map { Int($0.width.rounded()) },
            height: size.map { Int($0.height.rounded()) },
            durationMs: duration.isNumeric ? Int((duration.seconds * 1000).rounded()) : nil
        )
    }
}

private func writeJPEG(_ image: CGImage, to dest: String) throws {
    let url = URL(fileURLWithPath: dest) as CFURL
    guard let out = CGImageDestinationCreateWithURL(url, UTType.jpeg.identifier as CFString, 1, nil) else {
        throw FFIError.platform("cannot write thumbnail")
    }
    CGImageDestinationAddImage(out, image, [kCGImageDestinationLossyCompressionQuality: 0.8] as CFDictionary)
    guard CGImageDestinationFinalize(out) else { throw FFIError.platform("cannot write thumbnail") }
}

/// JPEG thumbnail no larger than `maxPixels` on its longest side, for images or videos.
func makeThumbnail(_ path: String, dest: String, maxPixels: Int) throws {
    let url = URL(fileURLWithPath: path)
    let isVideo = ["mp4", "mov", "m4v"].contains(url.pathExtension.lowercased())
    let image: CGImage
    if isVideo {
        let asset = AVURLAsset(url: url)
        let generator = AVAssetImageGenerator(asset: asset)
        generator.appliesPreferredTrackTransform = true
        generator.maximumSize = CGSize(width: maxPixels, height: maxPixels)
        generator.requestedTimeToleranceBefore = .positiveInfinity
        generator.requestedTimeToleranceAfter = .positiveInfinity
        image = try blocking {
            let duration = try await asset.load(.duration)
            // A frame a little in (first frames are often a still desktop or the countdown fade).
            let t = duration.isNumeric ? CMTime(seconds: min(1, duration.seconds / 3), preferredTimescale: 600) : .zero
            return try await generator.image(at: t).image
        }
    } else {
        // ImageIO decodes straight to the target size: fast and light on memory.
        guard let src = CGImageSourceCreateWithURL(url as CFURL, nil),
              let thumb = CGImageSourceCreateThumbnailAtIndex(src, 0, [
                  kCGImageSourceCreateThumbnailFromImageAlways: true,
                  kCGImageSourceCreateThumbnailWithTransform: true,
                  kCGImageSourceThumbnailMaxPixelSize: maxPixels,
              ] as CFDictionary)
        else { throw FFIError.platform("cannot read image") }
        image = thumb
    }
    try writeJPEG(image, to: dest)
}

@_cdecl("fl_media_info")
public func fl_media_info(_ path: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let p = String(cString: path)
    return respond { try videoInfo(p) }
}

@_cdecl("fl_thumbnail")
public func fl_thumbnail(_ path: UnsafePointer<CChar>, _ dest: UnsafePointer<CChar>, _ maxPixels: Int32) -> UnsafeMutablePointer<CChar> {
    let (p, d) = (String(cString: path), String(cString: dest))
    return respond { () throws -> Bool in
        try makeThumbnail(p, dest: d, maxPixels: Int(maxPixels))
        return true
    }
}
