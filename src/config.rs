use std::env;

pub struct Config {
  // Base Vars
  pub local_port: i32,
  pub remote_port: i32,
  pub remote_host: String,
  pub bind_addr: String,
  pub debug: bool,
  // Hooks Vars
  pub uri: String,
  pub token: String,
  pub service_id: String,
  pub time_before_stop: u64,
  pub time_tick_check_stop: u64,
  pub time_wait_status_change: u64,
  pub time_check_already_started: u64,
}

impl Config {
  pub fn new() -> Result<Config, &'static str> {
    // Retrieve values from environment variables or use default values
    let local_port = env::var("LOCAL_PORT").map_err(|_| "LOCAL_PORT environment variable not found")?.parse().map_err(|_| "Invalid value for LOCAL_PORT")?;
    let remote_port = env::var("REMOTE_PORT").map_err(|_| "REMOTE_PORT environment variable not found")?.parse().map_err(|_| "Invalid value for REMOTE_PORT")?;
    let remote_host = env::var("REMOTE_HOST").map_err(|_| "REMOTE_HOST environment variable not found")?;
    let bind_addr = env::var("BIND_ADDR").map_err(|_| "BIND_ADDR environment variable not found")?;
    let debug = env::var("DEBUG").map_err(|_| "DEBUG environment variable not found").unwrap_or("false".to_string()).parse().unwrap_or(false);

    let uri = env::var("URI").map_err(|_| "URI environment variable not found")?;
    let token = env::var("TOKEN").map_err(|_| "TOKEN environment variable not found")?;
    let service_id = env::var("SERVICE_ID").map_err(|_| "SERVICE_ID environment variable not found")?;
    let time_before_stop = env::var("TIME_BEFORE_STOP").unwrap_or("100".to_string()).parse().unwrap_or(300);
    let time_tick_check_stop = env::var("TIME_TICK_CHECK_STOP").unwrap_or("200".to_string()).parse().unwrap_or(300);
    let time_wait_status_change = env::var("TIME_WAIT_STATUS_CHANGE").unwrap_or("300".to_string()).parse().unwrap_or(400);
    let time_check_already_started = env::var("TIME_CHECK_ALREADY_STARTED").unwrap_or("400".to_string()).parse().unwrap_or(600);

    // Check for valid values, add more checks as needed
    if uri.is_empty() || token.is_empty() || service_id.is_empty() {
        return Err("URI, token, and service ID must not be empty");
    }

    // If all checks pass, return a Config object
    Ok(Config {
      local_port,
      remote_port,
      remote_host,
      bind_addr,
      debug,
      uri,
      token,
      service_id,
      time_before_stop,
      time_tick_check_stop,
      time_wait_status_change,
      time_check_already_started,
    })
  }
}