mod custom_func;

use std::error::Error;
use std::time::{Duration, SystemTime};

use crate::config::Config;

static mut SERVER_STARTED: bool = false;

static mut LAST_TIME_QUERY: Option<SystemTime> = None;

pub async fn hook_on_lauch() -> Result<(), Box<dyn Error>> {
	println!("Launched");

	tokio::spawn(async {
		let config = Config::new().map_err(|_| "Config anavailable").unwrap();
		let mut interval = tokio::time::interval(Duration::from_secs(config.time_tick_check_stop));

		loop {
			interval.tick().await;

			custom_func::check_no_traffic(&config).await.unwrap();
		}
	});
	Ok(())
}

pub async fn hook_on_packet_received(config: &Config) -> Result<(), Box<dyn Error>> {
  println!("Packet received");
	let now = SystemTime::now();

	unsafe {
		if LAST_TIME_QUERY.is_none() {
			custom_func::check_service_status(config).await.unwrap();
			LAST_TIME_QUERY = Some(now);
		}
		else {
			let elapsed = now.duration_since(LAST_TIME_QUERY.unwrap()).unwrap();
			println!("Time: {:?}", LAST_TIME_QUERY.unwrap());
			println!("Elapsed: {:?}", elapsed);

			if elapsed.as_secs() > 20 {
				LAST_TIME_QUERY = Some(now);

				if SERVER_STARTED == false || elapsed.as_secs() > 600 {
					custom_func::check_service_status(config).await.unwrap();
				}
			}
		}
	}
	Ok(())
}
