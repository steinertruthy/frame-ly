import CoreMedia

/// Maps capture timestamps (host clock) to output timestamps with paused intervals removed.
///
/// Pausing never stops the capture stream (so resuming is instant); samples that fall inside
/// a pause are dropped and every later sample is shifted back by the total paused time.
/// Works per-sample, so late-arriving buffers from before a pause are still placed correctly.
struct PauseTimeline {
    private(set) var intervals: [(start: CMTime, end: CMTime)] = []
    private(set) var pausedAt: CMTime?

    var isPaused: Bool { pausedAt != nil }

    mutating func pause(at t: CMTime) {
        guard pausedAt == nil else { return }
        pausedAt = t
    }

    mutating func resume(at t: CMTime) {
        guard let start = pausedAt else { return }
        if t > start { intervals.append((start, t)) }
        pausedAt = nil
    }

    /// Output time for `pts`, or nil if the sample was captured while paused.
    func adjusted(_ pts: CMTime) -> CMTime? {
        if let p = pausedAt, pts >= p { return nil }
        var shift = CMTime.zero
        for i in intervals {
            if pts >= i.start && pts < i.end { return nil }
            if pts >= i.end { shift = shift + (i.end - i.start) }
        }
        return pts - shift
    }

    /// Total paused duration up to host time `t` (for the elapsed-time display).
    func pausedDuration(until t: CMTime) -> CMTime {
        var total = intervals.reduce(CMTime.zero) { $0 + ($1.end - $1.start) }
        if let p = pausedAt, t > p { total = total + (t - p) }
        return total
    }
}
