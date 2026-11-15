//! FrameLy AI model catalog: what each profile and precision level uses, where it comes from,
//! its exact size and SHA-256, and its license. Chosen from a real evaluation on a pt-BR meeting
//! recording (see docs/architecture.md › FrameLy AI). Bumping `version` (with new file, size and
//! hash) offers the update to people who have the previous one.

use crate::core::settings::{AiProfile, Precision};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Speech recognition (whisper.cpp).
    Speech,
    /// Summaries and topics (llama.cpp, in the framely-llm process).
    Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    /// Stable role id; the UI translates it (`ai.model.<id>`).
    pub id: &'static str,
    pub kind: Kind,
    pub version: u32,
    pub file: &'static str,
    #[serde(skip)]
    pub url: &'static str,
    pub size: u64,
    #[serde(skip)]
    pub sha256: &'static str,
    pub license: &'static str,
    /// Approximate memory while in use.
    pub ram_mb: u32,
    /// Context window FrameLy uses (language models).
    pub context: u32,
}

pub const MODELS: &[ModelSpec] = &[
    ModelSpec {
        id: "speech-fast",
        kind: Kind::Speech,
        version: 1,
        file: "whisper-small-q5_1.v1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin",
        size: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        license: "MIT",
        ram_mb: 600,
        context: 0,
    },
    ModelSpec {
        id: "speech-balanced",
        kind: Kind::Speech,
        version: 1,
        file: "whisper-large-v3-turbo-q5_0.v1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin",
        size: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        license: "MIT",
        ram_mb: 1_100,
        context: 0,
    },
    ModelSpec {
        id: "speech-max",
        kind: Kind::Speech,
        version: 1,
        file: "whisper-large-v3-q5_0.v1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin",
        size: 1_081_140_203,
        sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1",
        license: "MIT",
        ram_mb: 2_200,
        context: 0,
    },
    ModelSpec {
        id: "language-basic",
        kind: Kind::Language,
        version: 1,
        file: "gemma-4-e2b-it-q4_0.v1.gguf",
        url: "https://huggingface.co/ggml-org/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_0.gguf",
        size: 2_841_481_184,
        sha256: "8e30dff3ac4c8434c49a7036fa15564bdbb6044e42bf04550bf1a096ad7e6a52",
        license: "Apache 2.0",
        ram_mb: 2_800,
        context: 8_192,
    },
    ModelSpec {
        id: "language-balanced",
        kind: Kind::Language,
        version: 1,
        file: "qwen3.5-4b-q4_k_m.v1.gguf",
        url: "https://huggingface.co/ggml-org/Qwen3.5-4B-GGUF/resolve/main/Qwen3.5-4B-Q4_K_M.gguf",
        size: 3_032_091_584,
        sha256: "217002fa8008ef4fc6d548353a3aebef055a96e6ff8e0e2c0953ef8254a89c14",
        license: "Apache 2.0",
        ram_mb: 3_600,
        context: 16_384,
    },
    ModelSpec {
        id: "language-advanced",
        kind: Kind::Language,
        version: 1,
        file: "qwen3.5-9b-q4_k_m.v1.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/main/Qwen3.5-9B-Q4_K_M.gguf",
        size: 5_680_522_464,
        sha256: "03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8",
        license: "Apache 2.0",
        ram_mb: 6_000,
        context: 32_768,
    },
];

pub fn spec(id: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|m| m.id == id)
}

/// Concrete precision for a profile when the setting is `Auto`.
pub fn precision_for(profile: AiProfile, precision: Precision) -> Precision {
    match precision {
        Precision::Auto => match profile {
            AiProfile::Basic => Precision::Fast,
            AiProfile::Advanced => Precision::Max,
            _ => Precision::Balanced,
        },
        p => p,
    }
}

pub fn speech_model(precision: Precision) -> &'static ModelSpec {
    spec(match precision {
        Precision::Fast => "speech-fast",
        Precision::Max => "speech-max",
        _ => "speech-balanced",
    })
    .expect("catalog")
}

pub fn language_model(profile: AiProfile) -> &'static ModelSpec {
    spec(match profile {
        AiProfile::Basic => "language-basic",
        AiProfile::Advanced => "language-advanced",
        _ => "language-balanced",
    })
    .expect("catalog")
}

pub fn precision_name(p: Precision) -> &'static str {
    match p {
        Precision::Fast => "fast",
        Precision::Max => "max",
        _ => "balanced",
    }
}

pub fn parse_precision(s: &str) -> Precision {
    match s {
        "fast" => Precision::Fast,
        "max" => Precision::Max,
        "balanced" => Precision::Balanced,
        _ => Precision::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_consistent() {
        for m in MODELS {
            assert_eq!(m.sha256.len(), 64, "{}", m.id);
            assert!(m.sha256.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(m.url.starts_with("https://huggingface.co/"));
            assert!(m.file.contains(&format!(".v{}.", m.version)), "file name carries the version: {}", m.id);
            assert!(m.size > 50_000_000);
            assert_eq!(m.kind == Kind::Language, m.context > 0);
        }
        let ids: std::collections::HashSet<_> = MODELS.iter().map(|m| m.id).collect();
        assert_eq!(ids.len(), MODELS.len());
    }

    #[test]
    fn profiles_map_to_models() {
        assert_eq!(precision_for(AiProfile::Basic, Precision::Auto), Precision::Fast);
        assert_eq!(precision_for(AiProfile::Advanced, Precision::Auto), Precision::Max);
        assert_eq!(precision_for(AiProfile::Basic, Precision::Max), Precision::Max, "explicit choice wins");
        assert_eq!(speech_model(Precision::Balanced).id, "speech-balanced");
        assert_eq!(language_model(AiProfile::Auto).id, "language-balanced");
        assert_eq!(language_model(AiProfile::Basic).id, "language-basic");
    }
}
