//! Platform-independent application core: errors, settings, paths, logging and locale.
//! Nothing in here calls OS capture APIs; those live in `platform/` (from Stage 1).

pub mod errors;
pub mod image;
pub mod locale;
pub mod logging;
#[allow(dead_code)] // consumed by the capture pipeline from Stage 2
pub mod paths;
pub mod settings;
