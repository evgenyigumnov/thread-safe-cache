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



impl<K: std::marker::Send  + 'static + Clone +  Eq + Hash, V: std::marker::Send  + Clone + 'static> ThreadSafeCache<K, V> {
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
                if !ret_clone.clean() {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
        });
        ret
    }

    pub fn put(&mut self, key: K, val: V)
        where K: Eq + Hash,
    {
        let mut md = self.implementation.lock().unwrap();
        if !md.cache.contains_key(&key) {
            md.current_size = md.current_size + 1;
        }
        md.cache.insert(key, val);
    }
    pub fn put_exp(&mut self, key: K, val: V, expiration: i32)
        where K: Eq + Hash + Clone,
    {
        let mut md = self.implementation.lock().unwrap();
        if !md.cache.contains_key(&key) {
            md.current_size = md.current_size + 1;
        }
        md.cache.insert(key.clone(), val);
        let now = std::time::SystemTime::now();
        let milliseconds_from_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i32 + expiration;
        md.expiration_set.entry(milliseconds_from_now)
            .and_modify(|curr| curr.push(key.clone())).or_insert({
            let mut ret = Vec::new();
            ret.push(key);
            ret
        });
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
        // println!("cleaning");
        let mut md = self.implementation.lock().unwrap();

        let now = std::time::SystemTime::now();
        let milliseconds_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i32;
        let new_expiration_set = md.expiration_set.split_off(&milliseconds_now);
        let old_clone = md.expiration_set.clone();
        // println!("old_clone: {}", old_clone.len());

        for (_, keys) in old_clone.iter() {
            // println!("keys len: {}", keys.len());
            for key in keys {
                let r = md.cache.remove(&key);
                if r.is_some() {
                    md.current_size = md.current_size - 1;
                }
            }
        }
        md.expiration_set = new_expiration_set;


        let count = Arc::strong_count(&self.implementation);
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
                cache1.put_exp("b", 2, 1000);
            });
            let mut cache2 = cache_init.clone();
            let t = thread::spawn(move || {
                sleep(Duration::from_millis(100));
                let ret = cache2.get("a");
                ret
            });
            assert_eq!(t.join().unwrap(), Some(1));

            thread::sleep(Duration::from_millis(100));
            let mut cache3 = cache_init.clone();
            assert_eq!(cache3.get("b"), Some(2));
            thread::sleep(Duration::from_millis(3000));
            assert_eq!(cache3.get("b"), None);

        }

        thread::sleep(Duration::from_millis(1000));

    }
}






