use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};


pub struct ThreadSafeCache<K, V> {
    cache: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> ThreadSafeCache<K, V> {
    pub fn new() -> ThreadSafeCache<K, V> {
        let amc: Arc<Mutex<HashMap<K, V>>> = Arc::new(Mutex::new(HashMap::new()));
        ThreadSafeCache {
            cache: amc
        }
    }

    pub fn put(&mut self, key: K, val: V)
        where K: Eq + Hash,
    {
        let mut md = self.cache.lock().unwrap();
        md.insert(key, val);
    }
    pub fn get(&mut self, key: K) -> Option<V>
        where K: Eq + Hash, V: Clone
    {
        let md = self.cache.lock().unwrap();
        md.get(&key).map(|s| s.clone())
    }
    pub fn rm(&mut self, key: K)
        where K: Eq + Hash,
    {
        let mut md = self.cache.lock().unwrap();
        md.remove(&key);
    }
}

impl<K, V> Clone for ThreadSafeCache<K, V> {
    fn clone(&self) -> ThreadSafeCache<K, V> {
        ThreadSafeCache {
            cache: Arc::clone(&self.cache)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    use super::*;

    #[test]
    fn it_works() {
        let cache_init: ThreadSafeCache<&str, i32> = ThreadSafeCache::new();
        let mut cache1 = cache_init.clone();
        thread::spawn(move || {
            cache1.put("a", 1);
        });
        let mut cache2 = cache_init.clone();
        let t = thread::spawn(move || {
            sleep(Duration::from_millis(100));
            cache2.rm("b");
            cache2.get("a")
        });
        assert_eq!(t.join().unwrap(), Some(1));
    }
}






