use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use thread_safe_cache::BuilderEmbedded;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut builder: BuilderEmbedded<String, i32> = BuilderEmbedded::init();
    builder.max_size(100);
    let cache_init = builder.build();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {

        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {

            let mut buf = [0; 1024];

            loop {
                println!(".");
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => {
                        n
                    },
                    Err(e) => {
                        return;
                    }
                };

                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    return;
                }
            }
        });
    }

}