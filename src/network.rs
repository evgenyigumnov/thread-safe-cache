use std::hash::Hash;
use tokio::net::TcpStream;
use crate::{ThreadSafeCacheTrait};
use serde_derive::{Serialize,Deserialize};
use std::io;
use tokio::io::AsyncWriteExt;

pub struct NetworkCache<K: Eq + Hash + serde::de::DeserializeOwned, V: serde::de::DeserializeOwned> {
    pub tcp_stream: TcpStream,
    pub rt: tokio::runtime::Runtime,
    pub phantom_data: std::marker::PhantomData<(K, V)>,

}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::Serialize + serde::de::DeserializeOwned +'static> ThreadSafeCacheTrait<K, V> for NetworkCache<K, V> {
    fn put(&mut self, key: K, val: V)
        where K: Eq + Hash,
    {
       self.rt.block_on(async {
            self.tcp_stream.writable().await.unwrap();
            let params = PutOpParams {
                key: key,
                val: val,
            };
           let mut encoded: Vec<u8> = bincode::serialize(&params).unwrap();
           let mut op_code:Vec<u8> = vec![CacheOp::Put as u8];
           op_code.append(&mut encoded);
           self.tcp_stream.write_all(op_code.as_slice());
           self.tcp_stream.readable().await.unwrap();
           let mut buf = Vec::with_capacity(4096);
           loop {
               match self.tcp_stream.try_read_buf(&mut buf) {
                   Ok(0) => {
                       panic!("closed");
                   },
                   Ok(n) => {
                       // println!(".{}",n);
                       break;
                   }
                   Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                       continue;
                   }
                   Err(e) => {
                       panic!("{}",e);
                   }
               }
           }

        });
    }
    fn put_exp(&mut self, key: K, val: V, expiration: i32)
        where K: Eq + Hash + Clone,
    {
    }
    fn get(&mut self, key: K) -> Option<V>
        where K: Eq + Hash, V: Clone
    {

        let ret = self.rt.block_on(async {
            self.tcp_stream.writable().await.unwrap();
            let params = GetOpParams {
                key: key,
            };
            let mut encoded: Vec<u8> = bincode::serialize(&params).unwrap();
            let mut op_code:Vec<u8> = vec![CacheOp::Get as u8];
            op_code.append(&mut encoded);

            self.tcp_stream.write_all(op_code.as_slice()).await;

            self.tcp_stream.readable().await.unwrap();
            let mut buf = Vec::with_capacity(4096);
            let mut ret:Option<V>;
            loop {
                match self.tcp_stream.try_read_buf(&mut buf) {
                    Ok(0) => {
                        panic!("closed");
                    },
                    Ok(n) => {
                        let ret_get: GetRet<V> = bincode::deserialize(&buf[0..n]).unwrap();
                        ret = ret_get.val;
                        break;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        panic!("{}",e);
                    }
                }
            }
            ret
        });
        ret
    }
    fn exists(&mut self, key: K) -> bool
        where K: Eq + Hash, V: Clone
    {
        false
    }
    fn rm(&mut self, key: K)
        where K: Eq + Hash,
    {
    }

}

pub enum CacheOp {
    Put = 1,
    PutExp = 2,
    Get = 3,
    Exists = 4,
    Rm = 5,
}

#[derive(Serialize, Deserialize)]
pub struct PutOpParams<K,V> {
    pub key: K,
    pub val: V
}
#[derive(Serialize, Deserialize)]
pub struct GetOpParams<K> {
    pub key: K
}
#[derive(Serialize, Deserialize)]
pub struct GetRet<V> {
    pub val: Option<V>,
}