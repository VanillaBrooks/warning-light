mod logging;
mod matrix;
#[cfg(feature = "pi")]
mod pi;

use std::thread::sleep;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;

#[macro_use]
extern crate log;

const SECONDS_BETWEEN_POLL: u64 = 20;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    println!("starting");

    let logging_path: String = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("failed to fetch logging path output from command line"))?;

    println!("fetched logging path");

    logging::setup_logs(&logging_path)
        .with_context(|| format!("failed to setup logs to {logging_path}"))?;

    #[cfg(feature = "pi")]
    let mut pi = pi::Pi::new().unwrap();

    info!("setting up matrix connection");
    let conn = matrix::MatrixConnection::new().await.unwrap();

    info!("fetching rooms to monitor");
    let mut monitor_rooms = conn.load_rooms().await.unwrap();

    info!("starting to poll");
    loop {
        let light_enable_seconds =
            matrix::poll_all_rooms(&conn, monitor_rooms.as_mut_slice()).await;

        match light_enable_seconds {
            Ok(enable_seconds) => {
                if let Some(num_seconds) = enable_seconds {
                    info!("turning on light for {num_seconds} seconds");

                    #[cfg(feature = "pi")]
                    pi.activate_light(num_seconds.into())
                }
            }
            Err(e) => {
                error!("failed to poll all rooms: {e}");
            }
        }

        sleep(Duration::from_secs(SECONDS_BETWEEN_POLL));
    }
}
