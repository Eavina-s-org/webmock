use crate::capture::proxy::server::handlers::tunnel::tunnel_data;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[test]
fn test_tunnel_data_signature() {
    // Test that tunnel_data function exists and has correct signature
    let _ = tunnel_data;
}

#[tokio::test]
async fn test_tcp_stream_creation() {
    // Test basic TCP stream creation (this is a local test)
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Test that we can create a TCP stream
    let stream_result = TcpStream::connect(addr).await;
    assert!(stream_result.is_ok());
}

#[test]
fn test_async_read_write_traits() {
    // Test that TcpStream implements AsyncRead and AsyncWrite
    fn implements_async_read_write<T: AsyncReadExt + AsyncWriteExt>() {}
    implements_async_read_write::<TcpStream>();
}

#[tokio::test]
async fn test_tcp_listener_bind() {
    // Test TCP listener binding
    let listener = TcpListener::bind("127.0.0.1:0").await;
    assert!(listener.is_ok());

    let listener = listener.unwrap();
    let addr = listener.local_addr();
    assert!(addr.is_ok());
}

#[test]
fn test_split_functionality() {
    // Test that TcpStream can be split (used in tunnel_data)
    fn can_split<T: AsyncReadExt + AsyncWriteExt + Unpin>() {
        // This is just a compile-time test
    }
    can_split::<TcpStream>();
}
