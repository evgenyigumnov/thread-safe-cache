use std::collections::{BTreeMap, HashMap};
use std::collections::btree_map::OccupiedEntry;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
mod tests;
struct ThreadSafeCacheImpl<K, V> {
    cache: HashMap<K, (V, u128)>,
    expiration_set: BTreeMap<u128,Vec<K>>,
    max_size: i32,
    current_size: i32,
}

pub struct ThreadSafeCache<K, V> {
    implementation: Arc<Mutex<ThreadSafeCacheImpl<K, V>>>,
}


pub struct Builder<K, V> {
    max_size: i32,
    phantom_data: std::marker::PhantomData<(K, V)>,
}

trait BuilderTrait<K, V> {
    fn build(self) -> ThreadSafeCache<K, V>;
    fn max_size(&mut self, max_size: i32) -> &mut Self;
    fn init() -> Builder<K, V> {
        Builder {
            max_size: 1000,
            phantom_data: Default::default(),
        }
    }
}

impl <K: std::marker::Send  + 'static + Clone +  Eq + Hash, V: std::marker::Send  + Clone + 'static> BuilderTrait<K, V> for Builder<K, V>  {
    fn build(self) ->  ThreadSafeCache<K, V> {

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

    fn max_size(&mut self, max_size: i32) -> &mut Self {
        self.max_size = max_size;
        self
    }
    
}


impl<K: std::marker::Send  + 'static + Clone +  Eq + Hash, V: std::marker::Send  + Clone + 'static> ThreadSafeCache<K, V> {
    pub fn new() -> ThreadSafeCache<K, V> {
        let mut builder: Builder<K,V> =Builder::init();
        builder.max_size(1000);
        let cache_build = builder.build();
        cache_build
    }

    pub fn put(&mut self, key: K, val: V)
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
        md.current_size = md.current_size + 1;


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
    pub fn put_exp(&mut self, key: K, val: V, expiration: i32)
        where K: Eq + Hash + Clone,
    {
        let mut md = self.implementation.lock().unwrap();
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
    pub fn get(&mut self, key: K) -> Option<V>
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
    pub fn exists(&mut self, key: K) -> bool
        where K: Eq + Hash, V: Clone
    {
        let md = self.implementation.lock().unwrap();
        let ret = md.cache.contains_key(&key);
        ret
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
        let milliseconds_now  = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u128;
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






