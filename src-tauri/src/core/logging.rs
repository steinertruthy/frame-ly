//! Structured logging: JSON lines in `<app log dir>/framely.log.<date>` (daily rotation, 7 files)
//! plus human-readable stderr output in debug builds. Level via `FRAMELY_LOG` (default `info`).

use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init(log_dir: &Path) -> Option<WorkerGuard> {
    let _ = std::fs::create_dir_all(log_dir);
    let filter =
        || EnvFilter::try_from_env("FRAMELY_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    let file = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("framely")
        .filename_suffix("log")
        .max_log_files(7)
        .build(log_dir)
        .ok();

    let (file_layer, guard) = match file {
        Some(appender) => {
            let (nb, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_writer(nb)
                .with_filter(filter());
            (Some(layer), Some(guard))
        }
        None => (None, None),
    };

    let stderr_layer = cfg!(debug_assertions).then(|| fmt::layer().compact().with_filter(filter()));

    let _ = tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .try_init();
    guard
}
