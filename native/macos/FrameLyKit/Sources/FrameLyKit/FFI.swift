import Foundation

// C ABI boundary with Rust.
//
// Protocol: every exported function returns a heap-allocated, NUL-terminated UTF-8 JSON
// string — `{"ok": <value>}` or `{"err": {"code": "...", "message": "..."}}` — which the
// caller must release with `fl_free_string`. JSON keeps the boundary tiny, typed on both
// sides (Codable / serde) and free of manual struct layout. Listing calls are infrequent,
// so serialization cost is irrelevant; the media hot path (Stage 3+) uses a separate
// callback-based interface.

struct FFIError: Error, Encodable {
    let code: String
    let message: String

    static func permissionDenied(_ what: String) -> FFIError {
        FFIError(code: "permission_denied", message: what)
    }
    static func platform(_ message: String) -> FFIError {
        FFIError(code: "platform", message: message)
    }
}

private struct OkEnvelope<T: Encodable>: Encodable { let ok: T }
private struct ErrEnvelope: Encodable { let err: FFIError }

// Keys stay camelCase, matching serde `rename_all = "camelCase"` on the Rust side.
private let encoder = JSONEncoder()

func respond<T: Encodable>(_ body: () throws -> T) -> UnsafeMutablePointer<CChar> {
    let data: Data
    do {
        data = try encoder.encode(OkEnvelope(ok: try body()))
    } catch let e as FFIError {
        data = (try? encoder.encode(ErrEnvelope(err: e))) ?? Data()
    } catch {
        data = (try? encoder.encode(ErrEnvelope(err: .platform(String(describing: error))))) ?? Data()
    }
    return strdup(String(decoding: data, as: UTF8.self))!
}

@_cdecl("fl_free_string")
public func fl_free_string(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

/// Runs `work` on the main thread and waits up to `timeout` for it. Returns nil if the main
/// thread is busy or has no run loop (e.g. under a test harness) instead of deadlocking.
func onMain<T>(timeout: TimeInterval = 0.5, _ work: @escaping () -> T) -> T? {
    if Thread.isMainThread { return work() }
    let box = ResultBox<T>()
    let sem = DispatchSemaphore(value: 0)
    DispatchQueue.main.async {
        box.result = .success(work())
        sem.signal()
    }
    guard sem.wait(timeout: .now() + timeout) == .success else { return nil }
    return try? box.result?.get()
}

/// Bridges an async API into the synchronous FFI. Must be called off the main thread
/// (Rust calls these from a blocking worker), otherwise main-actor work would deadlock.
func blocking<T>(_ op: @escaping @Sendable () async throws -> T) throws -> T {
    precondition(!Thread.isMainThread, "blocking() must not run on the main thread")
    let box = ResultBox<T>()
    let sem = DispatchSemaphore(value: 0)
    Task.detached {
        do { box.result = .success(try await op()) } catch { box.result = .failure(error) }
        sem.signal()
    }
    sem.wait()
    return try box.result!.get()
}

private final class ResultBox<T>: @unchecked Sendable {
    var result: Result<T, Error>?
}
