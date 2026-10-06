use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream}; 

// Where clients connect, and the one backend we forward everything to.
const LISTEN_ADDR: &str = "127.0.0.1:8080";
const BACKEND_ADDR: &str = "127.0.0.1:3001";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    println!("listening on {LISTEN_ADDR}, forwarding to {BACKEND_ADDR}");

    loop {
        // Accept a client connection.
        let (mut inbound, peer) = listener.accept().await?;

        // Handle it concurrently so one slow client can't block others.
        tokio::spawn(async move {
            match TcpStream::connect(BACKEND_ADDR).await {
                Ok(mut outbound) => {
                    // Pump bytes both ways until either side closes.
                    if let Err(e) = copy_bidirectional(&mut inbound, &mut outbound).await {
                        eprintln!("connection from {peer} ended: {e}");
                    }
                }
                Err(e) => eprintln!("backend unreachable: {e}"),
            }
        });
    }
}