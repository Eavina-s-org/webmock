use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

pub async fn tunnel_data(
    client: Upgraded,
    server: TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Wrap the upgraded connection with TokioIo for compatibility
    let client_io = TokioIo::new(client);

    // Split both connections
    let (mut client_read, mut client_write) = tokio::io::split(client_io);
    let (mut server_read, mut server_write) = tokio::io::split(server);

    // Bidirectional data transfer
    let client_to_server = async { tokio::io::copy(&mut client_read, &mut server_write).await };

    let server_to_client = async { tokio::io::copy(&mut server_read, &mut client_write).await };

    // Run both directions concurrently
    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
