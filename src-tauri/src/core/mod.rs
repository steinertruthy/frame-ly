//! Platform-independent application core: errors, settings, paths, logging and locale.
//! Nothing in here calls OS capture APIs; those live in `platform/` (from Stage 1).

pub mod denoise;
pub mod errors;
pub mod image;
pub mod library;
pub mod locale;
pub mod logging;
pub mod media_timeline;
#[allow(dead_code)] // consumed by the capture pipeline from Stage 2
pub mod paths;
pub mod recorder;
pub mod settings;
pub mod shortcuts;
pub mod sounds;
pub mod video;
pub mod yuv;
