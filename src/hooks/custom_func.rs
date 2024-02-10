use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::Client;
use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use crate::config::Config;
use crate::hooks;

#[derive(Debug, Deserialize)]
pub struct ChartRelease {
	id: String,
	status: String,
}

pub async fn make_get_request_with_token(url: &str, token: &str) -> Result<ChartRelease, Box<dyn Error>> {
		// Create a reqwest Client
		let client = Client::new();

		// Prepare the request headers with the bearer token
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert(
				AUTHORIZATION,
				HeaderValue::from_str(&format!("Bearer {}", token))?,
		);

		// Make the HTTP GET request
		let response = client
				.get(url)
				.headers(headers)
				.send()
				.await?
				.text()
		.await?;

	let item: ChartRelease = serde_json::from_str(&response)?;

	if item.id == "palworld-server" {
		// println!("{:?}", item);
		return Ok(item);
	}
	return Ok({
		ChartRelease {
			id: "".to_string(),
			status: "".to_string(),
		}
	});
	// Ok(response)
}

pub async fn make_post_request_with_token(url: &str, token: &str, post_data: &str) -> Result<String, Box<dyn Error>> {
		// Create a reqwest Client
		let client = Client::new();

		// Prepare the request headers with the bearer token
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert(
				AUTHORIZATION,
				HeaderValue::from_str(&format!("Bearer {}", token))?,
		);

	let body_bytes = post_data.as_bytes();

	// Make the HTTP GET request
		let response = client
				.post(url)
				.headers(headers)
		.body(body_bytes.to_vec())
				.send()
				.await?
				.text()
		.await?;

	Ok(response)
}

pub async fn service_start_stop(config: &Config, start: bool) -> Result<(), Box<dyn Error>> {
	// The URL to the service status endpoint
	// let url: &str = "http://192.168.1.97/api/v2.0/chart/release/scale";

	let mut url = "".to_string();
	url.push_str(&config.uri);
	url.push_str("/chart/release/scale");

	let mut post_data = "".to_string();

	post_data.push_str("{\"release_name\":\"");
	post_data.push_str(&config.service_id);
	post_data.push_str("\",\"scale_options\":{\"replica_count\":");
	if start {
		post_data.push_str("1");
	} else {
		post_data.push_str("0");
	}
	post_data.push_str("}}");

	// Make the request and print the response
	let response = make_post_request_with_token(&url, &config.token, &post_data).await?;
	println!("Response: {}", response);

	if start {
		sleep(Duration::from_secs(20));
		unsafe {
			hooks::SERVER_STARTED = true;
		}
	} else {
		unsafe {
			hooks::SERVER_STARTED = false;
		}
	}

	Ok(())
}

pub async fn check_service_status(config: &Config) -> Result<(), Box<dyn Error>> {
	// The URL to the service status endpoint
	// let url = "http://192.168.1.97/api/v2.0/chart/release/id/palworld-server";
	let mut url = "".to_string();
	url.push_str(&config.uri);
	url.push_str("/chart/release/id/");
	url.push_str(&config.service_id);

	// Make the request and print the response
		let response = make_get_request_with_token(&url, &config.token).await?;
	println!("Response: {}", response.status);

	if response.status == "STOPPED" {
		service_start_stop(config, true).await?;
	} else {
		println!("Service is already running");
	}
	Ok(())
}

pub async fn check_no_traffic(config: &Config) -> Result<(), Box<dyn Error>> {
	let now = SystemTime::now();

	unsafe {
		if hooks::LAST_TIME_QUERY.is_none() == true {
			return Ok(());
		}
		else {
			let elapsed: Duration = now.duration_since(hooks::LAST_TIME_QUERY.unwrap()).unwrap();
			println!("No Traffic - Elapsed: {:?}", elapsed);

			if elapsed.as_secs() > 600 {
				service_start_stop(config, false).await?;
				// sleep(Duration::from_secs(30));
				// check_service_status().await.unwrap();
				hooks::LAST_TIME_QUERY = None;
			}
		}
	}
	return Ok(());
}
