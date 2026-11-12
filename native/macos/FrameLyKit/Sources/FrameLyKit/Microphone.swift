import AVFoundation

/// Microphone capture delivering 48 kHz stereo Float32 sample buffers with host-clock timestamps.
final class MicCapture: NSObject, AVCaptureAudioDataOutputSampleBufferDelegate, @unchecked Sendable {
    private let session = AVCaptureSession()
    private let output = AVCaptureAudioDataOutput()
    private let onBuffer: (CMSampleBuffer, CMTime) -> Void
    private let onFailure: () -> Void
    private var observer: NSObjectProtocol?
    let deviceName: String

    /// `deviceId` = AVCaptureDevice.uniqueID; nil or unknown → system default input.
    init(deviceId: String?, queue: DispatchQueue,
         onBuffer: @escaping (CMSampleBuffer, CMTime) -> Void, onFailure: @escaping () -> Void) throws {
        guard AVCaptureDevice.authorizationStatus(for: .audio) == .authorized else {
            throw FFIError.permissionDenied("microphone")
        }
        guard let device = deviceId.flatMap(AVCaptureDevice.init(uniqueID:)) ?? AVCaptureDevice.default(for: .audio) else {
            throw FFIError(code: "no_microphone", message: "no microphone available")
        }
        deviceName = device.localizedName
        self.onBuffer = onBuffer
        self.onFailure = onFailure
        super.init()
        let input = try AVCaptureDeviceInput(device: device)
        session.beginConfiguration()
        guard session.canAddInput(input), session.canAddOutput(output) else {
            throw FFIError.platform("microphone cannot be used")
        }
        session.addInput(input)
        output.audioSettings = [
            AVFormatIDKey: kAudioFormatLinearPCM,
            AVSampleRateKey: AudioMixer.sampleRate,
            AVNumberOfChannelsKey: 2,
            AVLinearPCMBitDepthKey: 32,
            AVLinearPCMIsFloatKey: true,
            AVLinearPCMIsNonInterleaved: false,
        ]
        output.setSampleBufferDelegate(self, queue: queue)
        session.addOutput(output)
        session.commitConfiguration()
        // Device unplugged or taken over: stop waiting for it, keep recording the rest.
        observer = NotificationCenter.default.addObserver(
            forName: AVCaptureSession.runtimeErrorNotification, object: session, queue: nil
        ) { [weak self] _ in self?.onFailure() }
    }

    func start() { session.startRunning() }

    func stop() {
        session.stopRunning()
        if let observer { NotificationCenter.default.removeObserver(observer) }
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sb: CMSampleBuffer, from connection: AVCaptureConnection) {
        // Session timestamps use its synchronization clock; convert to the host clock that
        // ScreenCaptureKit (video, system audio) uses so everything shares one timeline.
        var pts = sb.presentationTimeStamp
        if let clock = session.synchronizationClock {
            pts = CMSyncConvertTime(pts, from: clock, to: CMClockGetHostTimeClock())
        }
        onBuffer(sb, pts)
    }
}

/// Lightweight microphone level meter for the "Test microphone" control in Settings.
final class MicMonitor: NSObject, AVCaptureAudioDataOutputSampleBufferDelegate, @unchecked Sendable {
    private let session = AVCaptureSession()
    private let queue = DispatchQueue(label: "app.framely.mic-monitor")
    private let lock = NSLock()
    private var peak: Float = 0

    init(deviceId: String?) throws {
        guard AVCaptureDevice.authorizationStatus(for: .audio) == .authorized else {
            throw FFIError.permissionDenied("microphone")
        }
        guard let device = deviceId.flatMap(AVCaptureDevice.init(uniqueID:)) ?? AVCaptureDevice.default(for: .audio) else {
            throw FFIError(code: "no_microphone", message: "no microphone available")
        }
        super.init()
        let input = try AVCaptureDeviceInput(device: device)
        let output = AVCaptureAudioDataOutput()
        guard session.canAddInput(input), session.canAddOutput(output) else {
            throw FFIError.platform("microphone cannot be used")
        }
        session.addInput(input)
        output.setSampleBufferDelegate(self, queue: queue)
        session.addOutput(output)
        session.startRunning()
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sb: CMSampleBuffer, from connection: AVCaptureConnection) {
        guard let samples = PCM.interleavedStereo(sb) else { return }
        let p = samples.reduce(0) { max($0, abs($1)) }
        lock.lock()
        peak = max(p, peak * 0.85) // fast attack, smooth release
        lock.unlock()
    }

    func level() -> Float {
        lock.lock(); defer { lock.unlock() }
        let v = peak
        peak *= 0.85
        return v
    }

    func stop() { session.stopRunning() }
}

private let monitorLock = NSLock()
nonisolated(unsafe) private var monitor: MicMonitor?

/// Starts the test meter. `deviceId` may be NULL (default input). Returns NULL or error JSON.
@_cdecl("fl_mic_monitor_start")
public func fl_mic_monitor_start(_ deviceId: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>? {
    do {
        let m = try MicMonitor(deviceId: deviceId.map { String(cString: $0) })
        monitorLock.lock()
        monitor?.stop()
        monitor = m
        monitorLock.unlock()
        return nil
    } catch {
        return respond { () throws -> Int in throw error }
    }
}

@_cdecl("fl_mic_monitor_level")
public func fl_mic_monitor_level() -> Float {
    monitorLock.lock(); defer { monitorLock.unlock() }
    return monitor?.level() ?? 0
}

@_cdecl("fl_mic_monitor_stop")
public func fl_mic_monitor_stop() {
    monitorLock.lock()
    monitor?.stop()
    monitor = nil
    monitorLock.unlock()
}
