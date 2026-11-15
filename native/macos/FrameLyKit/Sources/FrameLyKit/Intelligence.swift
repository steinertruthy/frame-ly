import Foundation
#if canImport(FoundationModels)
import FoundationModels
#endif

// Apple Intelligence (Foundation Models, macOS 26+): optional on-device provider for summaries
// and topics. Prompts are written by the Rust side (shared with the other providers); this file
// runs them with guided generation, so the output always has the exact structure FrameLy reads.
// FoundationModels is weak-linked: on macOS 14/15 everything here reports "unavailable".

struct AIStatus: Encodable {
    let available: Bool
    /// "os_version" | "device_not_eligible" | "not_enabled" | "model_not_ready" | "unknown"
    let reason: String?
    /// Whether the transcript's language is supported (nil when not asked).
    let supportsLanguage: Bool?
    /// Context window in tokens (input + output).
    let contextTokens: Int
}

struct AIRequest: Decodable {
    /// "summary" | "topics"
    let task: String
    let instructions: String
    let prompt: String
    let maxTokens: Int?
    let language: String?
}

struct AIResponse: Encodable {
    let json: String
}

func aiStatus(language: String?) -> AIStatus {
    #if canImport(FoundationModels)
    if #available(macOS 26.0, *) {
        let model = SystemLanguageModel.default
        let supports = language.map { model.supportsLocale(Locale(identifier: $0)) }
        switch model.availability {
        case .available:
            return AIStatus(available: true, reason: nil, supportsLanguage: supports, contextTokens: 4096)
        case .unavailable(let why):
            let reason: String
            switch why {
            case .deviceNotEligible: reason = "device_not_eligible"
            case .appleIntelligenceNotEnabled: reason = "not_enabled"
            case .modelNotReady: reason = "model_not_ready"
            @unknown default: reason = "unknown"
            }
            return AIStatus(available: false, reason: reason, supportsLanguage: supports, contextTokens: 4096)
        }
    }
    #endif
    return AIStatus(available: false, reason: "os_version", supportsLanguage: nil, contextTokens: 0)
}

#if canImport(FoundationModels)
@available(macOS 26.0, *)
@Generable
struct GPoint: Encodable {
    @Guide(description: "One key point, faithful to the transcript")
    var text: String
    @Guide(description: "Timestamp mm:ss of the passage the point comes from")
    var time: String
}

@available(macOS 26.0, *)
@Generable
struct GSummary: Encodable {
    @Guide(description: "Short context paragraph")
    var overview: String
    var points: [GPoint]
}

@available(macOS 26.0, *)
@Generable
struct GTimed: Encodable {
    var text: String
    @Guide(description: "Timestamp mm:ss")
    var time: String
}

@available(macOS 26.0, *)
@Generable
struct GAction: Encodable {
    var text: String
    @Guide(description: "Person responsible, only if said; empty otherwise")
    var owner: String
    @Guide(description: "Deadline, only if said; empty otherwise")
    var due: String
    var time: String
}

@available(macOS 26.0, *)
@Generable
struct GSubtopic: Encodable {
    var title: String
    @Guide(description: "Timestamp mm:ss")
    var time: String
    var points: [String]
}

@available(macOS 26.0, *)
@Generable
struct GTopic: Encodable {
    var title: String
    @Guide(description: "Timestamp mm:ss where the topic starts")
    var time: String
    var subtopics: [GSubtopic]
}

@available(macOS 26.0, *)
@Generable
struct GTopics: Encodable {
    var topics: [GTopic]
    var decisions: [GTimed]
    var actions: [GAction]
    var dates: [GTimed]
    var numbers: [GTimed]
    @Guide(description: "Up to 12 relevant words or names that appear verbatim in the transcript")
    var keywords: [String]
}

@available(macOS 26.0, *)
func aiGenerate(_ req: AIRequest) throws -> AIResponse {
    let status = aiStatus(language: nil)
    guard status.available else { throw FFIError(code: "unsupported", message: status.reason ?? "unavailable") }
    let options = GenerationOptions(temperature: 0.2, maximumResponseTokens: req.maxTokens)
    let encoder = JSONEncoder()
    do {
        return try blocking {
            let session = LanguageModelSession(instructions: req.instructions)
            let data: Data
            if req.task == "topics" {
                data = try encoder.encode(try await session.respond(to: req.prompt, generating: GTopics.self, options: options).content)
            } else {
                data = try encoder.encode(try await session.respond(to: req.prompt, generating: GSummary.self, options: options).content)
            }
            return AIResponse(json: String(decoding: data, as: UTF8.self))
        }
    } catch let e as LanguageModelSession.GenerationError {
        switch e {
        case .exceededContextWindowSize:
            throw FFIError(code: "context_overflow", message: "context window exceeded")
        case .unsupportedLanguageOrLocale:
            throw FFIError(code: "unsupported", message: "language")
        case .guardrailViolation:
            throw FFIError(code: "provider", message: "Apple Intelligence declined this content")
        default:
            throw FFIError(code: "provider", message: String(describing: e))
        }
    }
}
#endif

@_cdecl("fl_ai_status")
public func fl_ai_status(_ language: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar> {
    let lang = language.map { String(cString: $0) }
    return respond { aiStatus(language: lang?.isEmpty == false ? lang : nil) }
}

@_cdecl("fl_ai_generate")
public func fl_ai_generate(_ request: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let raw = Data(String(cString: request).utf8)
    return respond { () throws -> AIResponse in
        let req = try JSONDecoder().decode(AIRequest.self, from: raw)
        #if canImport(FoundationModels)
        if #available(macOS 26.0, *) {
            return try aiGenerate(req)
        }
        #endif
        throw FFIError(code: "unsupported", message: "os_version")
    }
}
