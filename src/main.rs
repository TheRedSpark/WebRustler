use env_logger::{Builder, WriteStyle};
use log::{info, LevelFilter};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut builder = Builder::new();
    builder
        .filter(None, LevelFilter::Info)
        .write_style(WriteStyle::Always)
        .init();
    info!("Client started with version {}",VERSION);
}
