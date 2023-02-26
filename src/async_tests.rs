use tokio::net::TcpStream;
use std::error::Error;
use std::io;
use tokio::test;

#[test]
async fn test_client() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    loop {
        // Wait for the socket to be writable
        stream.writable().await?;

        // Try to write data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_write(b"hello world") {
            Ok(n) => {
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }

        }



    }

    Ok(())
}