use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::io::{stdin,stdout,Write};
use tokio::time::{self, Duration, Instant};
use tokio::runtime::Builder;
use num_cpus; 

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
    counter: Arc<AtomicUsize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut interval = time::interval(Duration::from_secs(1));
    let start_time = Instant::now();
    let mut last_tick_time = start_time;

    for _ in 0..num_requests {
        {
            let mut stats_lock = stats.lock().await;
            stats_lock.increment_total();
        }
        
        socket.send_to(message, &server_address).await?;

        {
            let mut stats_lock = stats.lock().await;
            stats_lock.increment_successful();
        }

        counter.fetch_add(1, Ordering::SeqCst);

        if Instant::now() - last_tick_time >= Duration::from_secs(1) {
            let elapsed_time = start_time.elapsed();
            let mut stats_lock = stats.lock().await;
            println!(
                "Success rate: {:} reqs",
                stats_lock.success_rate(elapsed_time)
            );
            println!("Total requests: {}", counter.load(Ordering::SeqCst));
            last_tick_time = Instant::now();
        }
    }

    Ok(())
}

fn validate_user_input(ip: &str) -> bool {
    let octets: Vec<&str> = ip.split(".").collect();

    if octets.len() != 4 {
        return false;
    }

    for octet in octets {
        match octet.parse::<u8>() {
            Ok(num) => {
                if num > 255 {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }

    true
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut s=String::new();

    print!("Enter the ip addr and port in this format: IP:PORT: ");

    let _=stdout().flush();

    stdin().read_line(&mut s).expect("Did not enter a correct ip address, if the error persists contact the developer or make a pull request :)");

    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }

    if !validate_user_input(s) {
        Ok(())
    }

    let server_address: SocketAddr = s.parse()?;
    let message = vec![0; 1024]; 
    let num_requests_per_instance = 62500000; 
    let num_instances = num_cpus::get(); 
    let stats = Arc::new(Mutex::new(Stats::new()));
    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];


    for _ in 0..num_instances {
        let server_address = server_address.clone();
        let message = message.clone(); 
        let stats = stats.clone();
        let counter = counter.clone();

        let handle = tokio::spawn(async move {
            if let Err(err) = make_udp_requests(server_address, &message, num_requests_per_instance, stats, counter).await {
                eprintln!("Server returned an error: {}", err);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
