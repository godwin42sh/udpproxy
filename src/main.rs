mod hooks;
mod config;

use rand;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::error::Error;
use tokio::runtime::Runtime;

use crate::config::Config;

const TIMEOUT: u64 = 3 * 60 * 100; // 3 minutes, after 10 rounds
static mut DEBUG: bool = false;

fn main() -> Result<(), Box<dyn Error>> {
	let rt = Runtime::new()?;

	println!("Starting UDP Proxy");
	rt.block_on(async {
		let config = Config::new()?;

		unsafe {
		  DEBUG = config.debug;
		}

		let local_port: &i32 = &config.local_port;
		let remote_port: &i32 = &config.remote_port;
		let remote_host = &config.remote_host;
		let bind_addr = &config.bind_addr;

		hooks::hook_on_lauch().await.unwrap();
	
		forward(bind_addr, local_port, remote_host, remote_port, &config).await;
		Ok(())
	})
}

fn debug(msg: String) {
  let debug: bool;
  unsafe {
	  debug = DEBUG;
  }

  if debug {
	  println!("{}", msg);
  }
}

async fn forward(bind_addr: &str, local_port: &i32, remote_host: &str, remote_port: &i32, config: &Config) {
	let local_addr = format!("{}:{}", bind_addr, local_port);
	let local = UdpSocket::bind(&local_addr).expect(&format!("Unable to bind to {}", &local_addr));
	println!("Listening on {}", local.local_addr().unwrap());

	let remote_addr = format!("{}:{}", remote_host, remote_port);

	let responder = local
		.try_clone()
		.expect(&format!("Failed to clone primary listening address socket {}",
						local.local_addr().unwrap()));
	let (main_sender, main_receiver) = channel::<(_, Vec<u8>)>();
	thread::spawn(move || {
		debug(format!("Started new thread to deal out responses to clients"));
		loop {
			let (dest, buf) = main_receiver.recv().unwrap();
			let to_send = buf.as_slice();
			responder
				.send_to(to_send, dest)
				.expect(&format!("Failed to forward response from upstream server to client {}",
				dest));
		}
	});

	let mut client_map = HashMap::new();
	let mut buf = [0; 64 * 1024];
	loop {
		let (num_bytes, src_addr) = local.recv_from(&mut buf).expect("Didn't receive data");

		//we create a new thread for each unique client
		let mut remove_existing = false;
		loop {
			debug(format!("Received packet from client {}", src_addr));

			hooks::hook_on_packet_received(config).await.unwrap();

			let mut ignore_failure = true;
			let client_id = format!("{}", src_addr);

			if remove_existing {
				debug(format!("Removing existing forwarder from map."));
				client_map.remove(&client_id);
			}

			let sender = client_map.entry(client_id.clone()).or_insert_with(|| {
				//we are creating a new listener now, so a failure to send should be treated as an error
				ignore_failure = false;

				let local_send_queue = main_sender.clone();
				let (sender, receiver) = channel::<Vec<u8>>();
				let remote_addr_copy = remote_addr.clone();
				thread::spawn(move|| {
					//regardless of which port we are listening to, we don't know which interface or IP
					//address the remote server is reachable via, so we bind the outgoing
					//connection to 0.0.0.0 in all cases.
					let temp_outgoing_addr = format!("0.0.0.0:{}", 1024 + rand::random::<u16>());
					debug(format!("Establishing new forwarder for client {} on {}", src_addr, &temp_outgoing_addr));
					let upstream_send = UdpSocket::bind(&temp_outgoing_addr)
						.expect(&format!("Failed to bind to transient address {}", &temp_outgoing_addr));
					let upstream_recv = upstream_send.try_clone()
						.expect("Failed to clone client-specific connection to upstream!");

					let mut timeouts: u64 = 0;
					let timed_out = Arc::new(AtomicBool::new(false));

					let local_timed_out = timed_out.clone();
					thread::spawn(move|| {
						let mut from_upstream = [0; 64 * 1024];
						upstream_recv.set_read_timeout(Some(Duration::from_millis(TIMEOUT + 100))).unwrap();
						loop {
							match upstream_recv.recv_from(&mut from_upstream) {
								Ok((bytes_rcvd, _)) => {
									let to_send = from_upstream[..bytes_rcvd].to_vec();
									local_send_queue.send((src_addr, to_send))
										.expect("Failed to queue response from upstream server for forwarding!");
								},
								Err(_) => {
									if local_timed_out.load(Ordering::Relaxed) {
										debug(format!("Terminating forwarder thread for client {} due to timeout", src_addr));
										break;
									}
								}
							};
						}
					});

					loop {
						match receiver.recv_timeout(Duration::from_millis(TIMEOUT)) {
							Ok(from_client) => {
								upstream_send.send_to(from_client.as_slice(), &remote_addr_copy)
									.expect(&format!("Failed to forward packet from client {} to upstream server!", src_addr));
								timeouts = 0; //reset timeout count
							},
							Err(_) => {
								timeouts += 1;
								if timeouts >= 10 {
									debug(format!("Disconnecting forwarder for client {} due to timeout", src_addr));
									timed_out.store(true, Ordering::Relaxed);
									break;
								}
							}
						};
					}
				});
				sender
			});

			let to_send = buf[..num_bytes].to_vec();
			match sender.send(to_send) {
				Ok(_) => {
					break;
				}
				Err(_) => {
					if !ignore_failure {
						panic!("Failed to send message to datagram forwarder for client {}",
							   client_id);
					}
					//client previously timed out
					debug(format!("New connection received from previously timed-out client {}",
								  client_id));
					remove_existing = true;
					continue;
				}
			}
		}
	}
}
