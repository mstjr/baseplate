use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

pub fn init() {
    let filter_str = if !cfg!(debug_assertions) {
        "backend=info,tower_http=info,sqlx=info"
    } else {
        "info"
    };

    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_str));

    let registry = Registry::default().with(filter_layer);

    let result = if !cfg!(debug_assertions) {
        let json_layer = fmt::layer().json().with_ansi(false).with_target(false);
        registry.with(json_layer).try_init()
    } else {
        let text_layer = fmt::layer().compact().with_ansi(true).with_target(false);
        registry.with(text_layer).try_init()
    };

    if let Err(e) = result {
        eprintln!("Failed to initialize logging: {}", e);
    }
}
