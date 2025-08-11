//! Unified logging initialization for native builds.
//!
//! This module provides a consistent logging interface using the `log` crate as the common API
//! and `tracing` as the backend when the `enable-tracing` feature is enabled.

// Standard logging using tracing as backend
#[cfg(feature = "enable-tracing")]
pub fn init() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

    // Respect RUST_LOG. Default to info if nothing set.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry().with(env_filter).with(
        fmt::layer()
            .with_target(false)
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            ),
    );

    // It's okay if someone else already set a subscriber (e.g., in tests or binaries).
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Bridge log crate to tracing so log::info!() calls work
    tracing_log::LogTracer::init().ok();
}

#[cfg(not(feature = "enable-tracing"))]
#[inline]
pub fn init() {
    // no-op when logging disabled
}
