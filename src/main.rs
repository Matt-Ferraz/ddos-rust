use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};

#[derive(Debug)]
struct Stats {
    total_requests: usize,
    successful_requests: usize,
}

impl Stats {
    fn new() -> Self {
        Stats {
            total_requests: 0,
            successful_requests: 0,
        }
    }

    fn increment_total(&mut self) {
        self.total_requests += 1;
    }

    fn increment_successful(&mut self) {
        self.successful_requests += 1;
    }

    fn success_rate(&self, duration: Duration) -> f64 {
        self.successful_requests as f64
    }
}

async fn make_udp_requests(
    server_address: SocketAddr,
    message: &[u8],
    num_requests: usize,
    stats: Arc<Mutex<Stats>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut interval = time::interval(Duration::from_secs(1));
    let start_time = Instant::now();
    let mut last_tick_time = start_time;
    let mut stats_lock = stats.lock().await;

    for _ in 0..num_requests {
        stats_lock.increment_total();
        socket.send_to(message, &server_address).await?;
        stats_lock.increment_successful();

        if Instant::now() - last_tick_time >= Duration::from_secs(1) {
            let elapsed_time = start_time.elapsed();
            println!(
                "Success rate: {:} reqs in {:.2}",
                stats_lock.success_rate(elapsed_time),
                elapsed_time.as_secs_f64()
            );
            last_tick_time = Instant::now();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let server_address: SocketAddr = "ip:41234".parse().unwrap();
    let message = b"Hell
        o UDP Server";
    let num_requests = 10000;
    let num_instances = 10;
    let stats = Arc::new(Mutex::new(Stats::new()));
    let mut handles = vec![];

    for _ in 0..num_instances {
        let server_address = server_address.clone();
        let message = message.to_vec();
        let stats = stats.clone();
        let handle = tokio::spawn(async move {
            if let Err(err) = make_udp_requests(server_address, &message, num_requests, stats).await
            {
                eprintln!("Error: {}", err);
            }
        });
        handles.push(handle);
    }

    //
    for handle in handles {
        let _ = handle.await;
    }
}
