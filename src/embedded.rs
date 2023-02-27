use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::io::Read;
use std::sync::{Arc, Mutex};
use crate::{ThreadSafeCachePersistTrait, ThreadSafeCacheTrait};


pub struct ThreadSafeCacheImpl<K: Eq + Hash, V > {
    pub cache: HashMap<K, (V, u128)>,
    pub expiration_set: BTreeMap<u128,Vec<K>>,
    pub max_size: i32,
    pub current_size: i32,
}

pub struct ThreadSafeCache<K: Eq + Hash + serde::de::DeserializeOwned, V: serde::de::DeserializeOwned> {
    pub implementation: Arc<Mutex<ThreadSafeCacheImpl<K, V>>>,
}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::Serialize + serde::de::DeserializeOwned +'static> ThreadSafeCachePersistTrait<K, V> for ThreadSafeCache<K, V> {

     fn save(&mut self, file_name: &str) {
        let cloned = {
            let md = self.implementation.lock().unwrap();
            md.cache.clone()
        };
        let encoded: Vec<u8> = bincode::serialize(&cloned).unwrap();
        let mut file = File::create(file_name).unwrap();
        file.write_all(&encoded).unwrap();

    }

     fn load(&mut self, file_name: &str) {
        let buf: HashMap<K, (V, u128)>;
        let mut file = File::open(file_name).unwrap();
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded).unwrap();
        buf = bincode::deserialize(&encoded[..]).unwrap();
        let mut md = self.implementation.lock().unwrap();
        md.cache = buf;
        md.current_size = md.cache.len() as i32;
    }

}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::Serialize + serde::de::DeserializeOwned +'static>ThreadSafeCacheTrait<K, V> for ThreadSafeCache<K, V> {
    fn put(&mut self, key: K, val: V)
        where K: Eq + Hash,
    {
        let mut md = self.implementation.lock().unwrap();

        if md.current_size == md.max_size {
            let last_opt = md.expiration_set.pop_first();
            if let Some((_, last)) = last_opt {
                for key in last {
                    md.cache.remove(&key);
                    md.current_size = md.current_size - 1;
                }
            }
        }
        if !md.cache.contains_key(&key) {
            md.current_size = md.current_size + 1;
        }

        let now = std::time::SystemTime::now();
        let year:u128 = 1000 * 60 * 60 * 24 * 365;
        let milliseconds_from_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() + year;
        md.cache.insert(key.clone(), (val, milliseconds_from_now));
        md.expiration_set.entry(milliseconds_from_now)
            .and_modify(|curr| curr.push(key.clone())).or_insert({
            let mut ret = Vec::new();
            ret.push(key);
            ret
        });

    }
    fn put_exp(&mut self, key: K, val: V, expiration: i32)
        where K: Eq + Hash + Clone,
    {
        let mut md = self.implementation.lock().unwrap();
        if md.current_size == md.max_size {
            let last_opt = md.expiration_set.pop_first();
            if let Some((_, last)) = last_opt {
                for key in last {
                    md.cache.remove(&key);
                    md.current_size = md.current_size - 1;
                }
            }
        }

        if !md.cache.contains_key(&key) {
            md.current_size = md.current_size + 1;
        }
        let now = std::time::SystemTime::now();
        let milliseconds_from_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()  + expiration as u128;
        md.cache.insert(key.clone(), (val, milliseconds_from_now));
        md.expiration_set.entry(milliseconds_from_now)
            .and_modify(|curr| curr.push(key.clone())).or_insert({
            let mut ret = Vec::new();
            ret.push(key);
            ret
        });
    }
    fn get(&mut self, key: K) -> Option<V>
        where K: Eq + Hash, V: Clone
    {
        let md = self.implementation.lock().unwrap();
        let ret = md.cache.get(&key).map(|s| s.clone());
        if ret.is_some() {
            let (val, expiration) = ret.unwrap();
            if expiration > 0 {
                let now = std::time::SystemTime::now();
                let milliseconds_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u128;
                // println!("{} {}", milliseconds_now, expiration);
                if milliseconds_now > expiration {
                    return None;
                }
            }
            return Some(val);
        } else {
            return None;
        }
    }
    fn exists(&mut self, key: K) -> bool
        where K: Eq + Hash, V: Clone
    {
        let md = self.implementation.lock().unwrap();
        let ret = md.cache.contains_key(&key);
        ret
    }
    fn rm(&mut self, key: K)
        where K: Eq + Hash,
    {
        let mut md = self.implementation.lock().unwrap();
        let r = md.cache.remove(&key);
        if r.is_some() {
            md.current_size = md.current_size - 1;
        }
    }



}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash + serde::Serialize + serde::de::DeserializeOwned,
    V: std::marker::Send  + Clone + serde::Serialize + serde::de::DeserializeOwned +'static>ThreadSafeCache<K, V> {
    pub fn clean(&mut self) -> bool
    {
        // println!("cleaning");
        let mut md = self.implementation.lock().unwrap();

        let now = std::time::SystemTime::now();
        let milliseconds_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u128;
        let new_expiration_set = md.expiration_set.split_off(&milliseconds_now);
        let keys: Vec<K> = md.expiration_set.iter().map(|(_, keys)| {
            keys.clone()
        }).flatten().collect();
        // println!("Keys len : {}", keys.len());
        for key in keys {
            let r = md.cache.remove(&key);
            if r.is_some() {
                md.current_size = md.current_size - 1;
            }
        }

        md.expiration_set = new_expiration_set;
        // println!("expiration_set len : {}", md.expiration_set.len());

        let count = Arc::strong_count(&self.implementation);
        if count == 0 {
            false
        } else {
            true
        }
        true
    }


}

impl<K: Eq + Hash + serde::de::DeserializeOwned + serde::Serialize, V: serde::de::DeserializeOwned + serde::Serialize> Clone for ThreadSafeCache<K, V> {
    fn clone(&self) -> ThreadSafeCache<K, V> {
        ThreadSafeCache {
            implementation: Arc::clone(&self.implementation),
        }
    }
}