use std::sync::Once;
use tracing_subscriber::EnvFilter;

static INIT: Once = Once::new();

pub fn tinnitus() -> anyhow::Result<()> {
    INIT.call_once(|| {
        #[cfg(feature = "dotenv")]
        if let Err(e) = dotenvy::dotenv() && !e.not_found() {
            eprintln!("Warning: could not load .env: {}", e);
        }

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    });

    Ok(())
}
