use log::debug;
use tokio::fs::OpenOptions;
use structured_logger::async_json::new_writer;
use structured_logger::Builder;
use crate::util::{Config};
use crate::task::load_gtfs::load_gtfs;

mod task;
mod model;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open("app.log")
        .await?;

    Builder::with_level("trace")
        .with_default_writer(new_writer(log_file))
        .init();

    let cfg: Config = confy::load("cta-monitor", None)?;
    debug!("Loaded configuration: {:?}", &cfg);

    load_gtfs(&cfg).await?;

    Ok(())
}
