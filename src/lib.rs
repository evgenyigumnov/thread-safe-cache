mod tests;
mod embedded;
mod network;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use crate::embedded::{ThreadSafeCache, ThreadSafeCacheImpl};
use crate::network::NetworkCache;


pub struct BuilderNetwork<K, V> {
    address: String,
    rt: tokio::runtime::Runtime,
    phantom_data: std::marker::PhantomData<(K, V)>,
}


impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::de::DeserializeOwned + serde::Serialize +  'static>  BuilderNetwork<K, V>  {
    pub fn init() -> BuilderNetwork<K, V> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        BuilderNetwork {
            address: "".to_string(),
            rt: rt,
            phantom_data: Default::default(),
        }
    }

    pub fn address(&mut self, address: String) -> &mut Self {
        self.address = address;
        self
    }
     pub fn connect(self) -> NetworkCache<K, V> {
         //         TcpStream::connect(self.address.as_str()).await.unwrap()
         let  stream = self.rt.block_on(async {
             let stream = TcpStream::connect(self.address.as_str()).await.unwrap();
             stream
         });
        let ret = NetworkCache {
            tcp_stream:  stream,
            rt: self.rt,
            phantom_data: Default::default(),
        };
        ret
    }

}

pub struct BuilderEmbedded<K, V> {
    max_size: i32,
    phantom_data: std::marker::PhantomData<(K, V)>,
}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::de::DeserializeOwned + serde::Serialize +  'static>  BuilderEmbedded<K, V>  {
    pub fn init() -> BuilderEmbedded<K, V> {
        BuilderEmbedded {
            max_size: 1000,
            phantom_data: Default::default(),
        }
    }

    pub fn build(self) -> ThreadSafeCache<K, V> {

        let im = Arc::new(Mutex::new(ThreadSafeCacheImpl {
            cache: HashMap::new(),
            expiration_set: BTreeMap::new(),
            max_size: self.max_size,
            current_size: 0,
        }));
        let ret = ThreadSafeCache {
            implementation: im,
        };
        let mut ret_clone = ret.clone();
        thread::spawn(move || {
            loop {
                if !ret_clone.clean() {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
        });
        ret

    }

    pub fn max_size(&mut self, max_size: i32) -> &mut Self {
        self.max_size = max_size;
        self
    }
    
}


pub trait  ThreadSafeCacheTrait<K: 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V:   Clone + serde::Serialize + serde::de::DeserializeOwned +'static> {
    fn put(&mut self, key: K, val: V)
        where K: Eq + Hash;
    fn put_exp(&mut self, key: K, val: V, expiration: i32)
        where K: Eq + Hash + Clone;
    fn get(&mut self, key: K) -> Option<V>
        where K: Eq + Hash, V: Clone;
    fn exists(&mut self, key: K) -> bool;
    fn rm(&mut self, key: K);
}
pub trait ThreadSafeCachePersistTrait<K:   'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V:  Clone + serde::Serialize + serde::de::DeserializeOwned +'static>: ThreadSafeCacheTrait<K,V>  {
    fn save(&mut self, file_name: &str);
    fn load(&mut self, file_name: &str);
}





