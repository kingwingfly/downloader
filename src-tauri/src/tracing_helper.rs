use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn init_tracing_subscriber() {
    tracing_subscriber::fmt()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_max_level(Level::DEBUG)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tracing::debug!("tracing started");
}
