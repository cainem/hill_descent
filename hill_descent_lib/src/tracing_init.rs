//! Unified logging initialization that works for both WASM and non-WASM builds.
//!
//! This module provides a consistent logging interface using the `log` crate as the common API,
//! with different backends for different targets:
//! - WASM builds: outputs to browser console via `console_log`
//! - Non-WASM builds: outputs to stdout/stderr via `tracing` with `tracing-log` bridge
//!
//! The code is entirely compiled out when logging features are not enabled.

// WASM-specific logging that outputs to browser console
#[cfg(all(
    target_arch = "wasm32",
    any(feature = "enable-tracing", feature = "wasm-logging")
))]
pub fn init() {
    console_log::init_with_level(log::Level::Info).expect("error initializing log");
}

// Standard logging for non-WASM builds using tracing as backend
#[cfg(all(not(target_arch = "wasm32"), feature = "enable-tracing"))]
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
    #[cfg(not(target_arch = "wasm32"))]
    tracing_log::LogTracer::init().ok();
}

#[cfg(not(any(feature = "enable-tracing", feature = "wasm-logging")))]
#[inline]
pub fn init() {
    // no-op when logging disabled
}
