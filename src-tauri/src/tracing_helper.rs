use tracing::Level;

pub fn init_tracing_subscriber() {
    tracing_subscriber::fmt()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_max_level(Level::DEBUG)
        .init();
}
