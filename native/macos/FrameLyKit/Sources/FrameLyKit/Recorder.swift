import AVFoundation
import VideoToolbox
import CoreImage
import ScreenCaptureKit

// Screen recording: ScreenCaptureKit stream → AVAssetWriter (VideoToolbox hardware encoder).
//
// - Output is a fragmented MP4 (a fragment every 2 s), so a crash or power loss leaves a
//   playable file up to the last fragment; Rust recovers those on next launch.
// - Pause/resume keep the stream running and retime samples (see PauseTimeline).
// - The stream is configured to deliver frames at most at `fps`; ScreenCaptureKit sends frames
//   only when content changes, so static screens cost almost nothing (variable frame rate).
// - Events go back to Rust through a C callback as JSON: {"kind": "started" | "stopped", ...}.

struct RecordingConfig: Decodable {
    struct Target: Decodable {
        let kind: String // "display" | "window" | "region"
        let displayId: String?
        let windowId: String?
        /// Display-local rectangle in points (region recordings).
        let x, y, width, height: Double?
    }
    let target: Target
    let width: Int
    let height: Int
    let fps: Int
    let bitrate: Int
    let codec: String // "h264" | "hevc"
    let showsCursor: Bool
    let outputPath: String
    let thumbnailPath: String
}

struct RecordingResult: Encodable {
    let durationMs: Int
    let frames: Int
    let width: Int
    let height: Int
}

public typealias EventCallback = @convention(c) (UnsafePointer<CChar>) -> Void

final class Recorder: NSObject, SCStreamOutput, SCStreamDelegate, @unchecked Sendable {
    private let config: RecordingConfig
    private let queue = DispatchQueue(label: "app.framely.recorder", qos: .userInteractive)
    private let emit: (String) -> Void
    private var stream: SCStream?
    private let writer: AVAssetWriter
    private let input: AVAssetWriterInput
    private var timeline = PauseTimeline()
    private var firstPTS: CMTime?
    private var lastPTS: CMTime = .zero
    private var frames = 0
    private var thumbnailDone = false
    private var finished = false

    init(config: RecordingConfig, emit: @escaping (String) -> Void) throws {
        self.config = config
        self.emit = emit
        let url = URL(fileURLWithPath: config.outputPath)
        try? FileManager.default.removeItem(at: url)
        writer = try AVAssetWriter(outputURL: url, fileType: .mp4)
        writer.movieFragmentInterval = CMTime(seconds: 2, preferredTimescale: 600)
        writer.shouldOptimizeForNetworkUse = false

        let hevc = config.codec == "hevc"
        var compression: [String: Any] = [
            AVVideoAverageBitRateKey: config.bitrate,
            AVVideoExpectedSourceFrameRateKey: config.fps,
            AVVideoMaxKeyFrameIntervalKey: config.fps * 2,
            AVVideoAllowFrameReorderingKey: false,
        ]
        compression[AVVideoProfileLevelKey] = hevc
            ? kVTProfileLevel_HEVC_Main_AutoLevel as String
            : AVVideoProfileLevelH264HighAutoLevel
        input = AVAssetWriterInput(mediaType: .video, outputSettings: [
            AVVideoCodecKey: hevc ? AVVideoCodecType.hevc : AVVideoCodecType.h264,
            AVVideoWidthKey: config.width,
            AVVideoHeightKey: config.height,
            AVVideoCompressionPropertiesKey: compression,
            AVVideoColorPropertiesKey: [
                AVVideoColorPrimariesKey: AVVideoColorPrimaries_ITU_R_709_2,
                AVVideoTransferFunctionKey: AVVideoTransferFunction_ITU_R_709_2,
                AVVideoYCbCrMatrixKey: AVVideoYCbCrMatrix_ITU_R_709_2,
            ],
        ])
        input.expectsMediaDataInRealTime = true
        guard writer.canAdd(input) else { throw FFIError.platform("encoder rejected the output settings") }
        writer.add(input)
        super.init()
    }

    func start() throws {
        guard CGPreflightScreenCaptureAccess() else { throw FFIError.permissionDenied("screen") }
        let content = try blocking { try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true) }
        let ownPid = ProcessInfo.processInfo.processIdentifier
        let ownApps = content.applications.filter { $0.processID == ownPid }

        let stream = SCStreamConfiguration()
        stream.width = config.width
        stream.height = config.height
        stream.minimumFrameInterval = CMTime(value: 1, timescale: CMTimeScale(config.fps))
        stream.pixelFormat = kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange
        stream.colorMatrix = CGDisplayStream.yCbCrMatrix_ITU_R_709_2
        stream.colorSpaceName = CGColorSpace.sRGB
        stream.showsCursor = config.showsCursor
        stream.queueDepth = 6
        stream.scalesToFit = true

        let filter: SCContentFilter
        let t = config.target
        switch t.kind {
        case "window":
            guard let w = content.windows.first(where: { String($0.windowID) == t.windowId }) else {
                throw FFIError.platform("window not found")
            }
            filter = SCContentFilter(desktopIndependentWindow: w)
        case "display", "region":
            guard let d = content.displays.first(where: { String($0.displayID) == t.displayId }) else {
                throw FFIError.platform("display not found")
            }
            filter = SCContentFilter(display: d, excludingApplications: ownApps, exceptingWindows: [])
            if t.kind == "region", let x = t.x, let y = t.y, let w = t.width, let h = t.height {
                stream.sourceRect = CGRect(x: x, y: y, width: w, height: h)
            }
        default:
            throw FFIError.platform("unknown target \(t.kind)")
        }

        let s = SCStream(filter: filter, configuration: stream, delegate: self)
        try s.addStreamOutput(self, type: .screen, sampleHandlerQueue: queue)
        guard writer.startWriting() else {
            throw FFIError.platform("could not start writing: \(writer.error?.localizedDescription ?? "unknown")")
        }
        try blocking { try await s.startCapture() }
        self.stream = s
    }

    // MARK: - Samples

    func stream(_ stream: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen, !finished, sb.isValid, isCompleteFrame(sb) else { return }
        let pts = sb.presentationTimeStamp
        guard let adjusted = timeline.adjusted(pts) else { return }

        if firstPTS == nil {
            firstPTS = adjusted
            writer.startSession(atSourceTime: adjusted)
            emit(#"{"kind":"started"}"#)
        }
        guard input.isReadyForMoreMediaData else { return } // encoder busy: drop rather than stall capture
        guard let retimed = retime(sb, to: adjusted), input.append(retimed) else { return }
        lastPTS = adjusted
        frames += 1
        if !thumbnailDone, let pb = sb.imageBuffer {
            thumbnailDone = true
            writeThumbnail(pb)
        }
    }

    private func isCompleteFrame(_ sb: CMSampleBuffer) -> Bool {
        guard let attachments = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
              let raw = attachments.first?[.status] as? Int,
              let status = SCFrameStatus(rawValue: raw)
        else { return false }
        return status == .complete
    }

    private func retime(_ sb: CMSampleBuffer, to pts: CMTime) -> CMSampleBuffer? {
        var timing = CMSampleTimingInfo(duration: sb.duration, presentationTimeStamp: pts, decodeTimeStamp: .invalid)
        var out: CMSampleBuffer?
        CMSampleBufferCreateCopyWithNewTiming(allocator: nil, sampleBuffer: sb, sampleTimingEntryCount: 1, sampleTimingArray: &timing, sampleBufferOut: &out)
        return out
    }

    private func writeThumbnail(_ pb: CVPixelBuffer) {
        let image = CIImage(cvPixelBuffer: pb)
        let path = config.thumbnailPath
        DispatchQueue.global(qos: .utility).async {
            let scale = min(1, 640 / image.extent.width)
            let scaled = image.transformed(by: CGAffineTransform(scaleX: scale, y: scale))
            let ctx = CIContext()
            try? ctx.writeJPEGRepresentation(
                of: scaled, to: URL(fileURLWithPath: path),
                colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!,
                options: [kCGImageDestinationLossyCompressionQuality as CIImageRepresentationOption: 0.8]
            )
        }
    }

    // MARK: - Control

    private static func hostNow() -> CMTime { CMClockGetTime(CMClockGetHostTimeClock()) }

    func pause() { queue.sync { timeline.pause(at: Self.hostNow()) } }
    func resume() { queue.sync { timeline.resume(at: Self.hostNow()) } }

    /// Stops capture and finalizes the file. Safe to call once; later calls are no-ops.
    func stop() throws -> RecordingResult {
        let stopTime = Self.hostNow()
        if let s = stream {
            stream = nil
            try? blocking { try await s.stopCapture() }
        }
        return try finalize(endingAt: stopTime)
    }

    private func finalize(endingAt hostTime: CMTime) throws -> RecordingResult {
        try queue.sync {
            guard !finished else { throw FFIError.platform("recording already finished") }
            finished = true
            guard let first = firstPTS, frames > 0 else {
                writer.cancelWriting()
                try? FileManager.default.removeItem(atPath: config.outputPath)
                throw FFIError(code: "empty_recording", message: "no frames were captured")
            }
            // Keep the last frame on screen until the moment the user pressed stop (static
            // screens deliver no new frames), minus any time spent paused.
            if timeline.isPaused { timeline.resume(at: hostTime) }
            let end = max(lastPTS, hostTime - timeline.pausedDuration(until: hostTime))
            input.markAsFinished()
            writer.endSession(atSourceTime: end)
            let sem = DispatchSemaphore(value: 0)
            writer.finishWriting { sem.signal() }
            sem.wait()
            guard writer.status == .completed else {
                throw FFIError.platform("could not finish the file: \(writer.error?.localizedDescription ?? "unknown")")
            }
            return RecordingResult(
                durationMs: Int(((end - first).seconds * 1000).rounded()),
                frames: frames,
                width: config.width,
                height: config.height
            )
        }
    }

    // MARK: - SCStreamDelegate

    /// The system stopped the stream (window closed, display unplugged, user clicked "Stop" in
    /// the menu bar, permission revoked). Finalize what we have and tell Rust.
    func stream(_ stream: SCStream, didStopWithError error: Error) {
        self.stream = nil
        let reason = (error as NSError).localizedDescription
        DispatchQueue.global().async { [self] in
            let result = try? finalize(endingAt: Self.hostNow())
            let payload: [String: Any] = [
                "kind": "stopped",
                "reason": reason,
                "durationMs": result?.durationMs ?? 0,
                "frames": result?.frames ?? 0,
            ]
            if let data = try? JSONSerialization.data(withJSONObject: payload) {
                emit(String(decoding: data, as: UTF8.self))
            }
        }
    }
}

// MARK: - C exports (one recording at a time)

private let lock = NSLock()
nonisolated(unsafe) private var current: Recorder?
nonisolated(unsafe) private var callback: EventCallback?

private func sendEvent(_ json: String) {
    guard let cb = callback else { return }
    json.withCString { cb($0) }
}

@_cdecl("fl_rec_set_callback")
public func fl_rec_set_callback(_ cb: EventCallback?) {
    lock.lock(); defer { lock.unlock() }
    callback = cb
}

@_cdecl("fl_rec_start")
public func fl_rec_start(_ configJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    respond { () throws -> Bool in
        lock.lock()
        let busy = current != nil
        lock.unlock()
        if busy { throw FFIError(code: "busy", message: "a recording is already in progress") }
        let config = try JSONDecoder().decode(RecordingConfig.self, from: Data(String(cString: configJson).utf8))
        let recorder = try Recorder(config: config, emit: sendEvent)
        try recorder.start()
        lock.lock(); current = recorder; lock.unlock()
        return true
    }
}

@_cdecl("fl_rec_pause")
public func fl_rec_pause() {
    lock.lock(); let r = current; lock.unlock()
    r?.pause()
}

@_cdecl("fl_rec_resume")
public func fl_rec_resume() {
    lock.lock(); let r = current; lock.unlock()
    r?.resume()
}

@_cdecl("fl_rec_stop")
public func fl_rec_stop() -> UnsafeMutablePointer<CChar> {
    respond { () throws -> RecordingResult in
        lock.lock(); let r = current; current = nil; lock.unlock()
        guard let r else { throw FFIError(code: "not_recording", message: "no recording in progress") }
        return try r.stop()
    }
}

/// Releases a recorder that the system already stopped (after a "stopped" event).
@_cdecl("fl_rec_discard")
public func fl_rec_discard() {
    lock.lock(); current = nil; lock.unlock()
}
