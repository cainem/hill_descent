//! Runtime initialization for tokio-tracing when the `enable-tracing` feature is active.
//!
//! The code is entirely compiled out when the feature is not enabled, so there is zero
//! runtime or compile-time overhead in that case.

#[cfg(feature = "enable-tracing")]
pub fn init() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

    // Respect RUST_LOG. Default to debug if nothing set.
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
}

#[cfg(not(feature = "enable-tracing"))]
#[inline]
pub fn init() {
    // no-op when tracing disabled
}
