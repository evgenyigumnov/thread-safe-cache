use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct ThreadSafeCacheImpl<K, V> {
    cache: HashMap<K, V>,
    expiration_set: BTreeMap<i32,Vec<K>>,
    max_size: i32,
    current_size: i32,
}

pub struct ThreadSafeCache<K, V> {
    implementation: Arc<Mutex<ThreadSafeCacheImpl<K, V>>>,
}



impl<K: std::marker::Send  + 'static, V: std::marker::Send  + 'static> ThreadSafeCache<K, V> {
    pub fn new() -> ThreadSafeCache<K, V> {
        let im = Arc::new(Mutex::new(ThreadSafeCacheImpl {
            cache: HashMap::new(),
            expiration_set: BTreeMap::new(),
            max_size: 100,
            current_size: 0,
        }));
        let ret = ThreadSafeCache {
            implementation: im,
        };
        let mut ret_clone = ret.clone();
        thread::spawn(move || {
            loop {
                println!("cleaning cache...");
                if !ret_clone.clean() {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        ret
    }

    pub fn put(&mut self, key: K, val: V)
        where K: Eq + Hash,
    {
        let mut md = self.implementation.lock().unwrap();
        md.cache.insert(key, val);
        md.current_size = md.current_size + 1;
    }
    pub fn get(&mut self, key: K) -> Option<V>
        where K: Eq + Hash, V: Clone
    {
        let md = self.implementation.lock().unwrap();
        md.cache.get(&key).map(|s| s.clone())
    }
    pub fn rm(&mut self, key: K)
        where K: Eq + Hash,
    {
        let mut md = self.implementation.lock().unwrap();
        let r = md.cache.remove(&key);
        if r.is_some() {
            md.current_size = md.current_size - 1;
        }
    }

    fn clean(&mut self) -> bool
    {
        let mut md = self.implementation.lock().unwrap();

        let count = Arc::strong_count(&self.implementation);
        println!("count: {}", count);
        if count == 1 {
             false
        } else {
            true
        }
    }



}

impl<K, V> Clone for ThreadSafeCache<K, V> {
    fn clone(&self) -> ThreadSafeCache<K, V> {
        ThreadSafeCache {
            implementation: Arc::clone(&self.implementation),
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

        {
            let cache_init: ThreadSafeCache<&str, i32> = ThreadSafeCache::new();

            let mut cache1 = cache_init.clone();
            thread::spawn(move || {
                cache1.put("a", 1);
            });
            let mut cache2 = cache_init.clone();
            let t = thread::spawn(move || {
                sleep(Duration::from_millis(100));
                cache2.rm("b");
                let ret = cache2.get("a");
                ret
            });
            assert_eq!(t.join().unwrap(), Some(1));

            thread::sleep(Duration::from_millis(1000));

        }
        println!("dead");

        thread::sleep(Duration::from_millis(1000));

    }
}






