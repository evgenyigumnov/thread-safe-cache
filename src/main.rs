extern crate core;

use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use thread_safe_cache::*;
use thread_safe_cache::network::{CacheOp, GetOpParams, PutOpParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder: BuilderEmbedded<String, i32> = BuilderEmbedded::init();
    builder.max_size(100);
    let mut cache_init = builder.build();
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let mut cache1 = cache_init.clone();
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let mut cache = cache1.clone();
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => {
                        let op = buf[0];
                        if op == CacheOp::Put as u8 {
                            let put_params:PutOpParams<String, i32> = bincode::deserialize(&buf[1..n]).unwrap();
                            cache.put(put_params.key, put_params.val);
                            // println!("put");
                            socket.write_all(b"ok").await;

                        }
                        if op == CacheOp::Get as u8 {
                            let get_params:GetOpParams<String> = bincode::deserialize(&buf[1..n]).unwrap();
                            let ret = cache.get(get_params.key);
                            let mut encoded: Vec<u8> = bincode::serialize(&ret).unwrap();
                            socket.write_all(encoded.as_slice()).await;
                            // println!("get");

                        }
                        // println!(".{}",n);
                        n
                    },
                    Err(e) => {
                        return;
                    }
                };
                // if let Err(e) = socket.write_all(&buf[0..n]).await {
                //     println!("e");
                //     return;
                // }
            }
        });
    }
}