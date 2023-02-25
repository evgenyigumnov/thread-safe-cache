# thread-safe-cache

```
use thread_safe_cache::*;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
fn main() {
    let cache_init: ThreadSafeCache<&str, i32> = ThreadSafeCache::new();
    let mut cache1 = cache_init.clone();
    thread::spawn(move || {
        cache1.put("key1", 1);
        cache1.put_exp("key2", 2, 3000);
    });
    let mut cache2 = cache_init.clone();
    let t = thread::spawn(move || {
        sleep(Duration::from_millis(2000));
        cache2.rm("key1");
        cache2.get("key2")
    });
    assert_eq!(t.join().unwrap(),Some(2));
    let mut cache3 = cache_init.clone();
    assert_eq!(cache3.get("key2"), Some(2));
    sleep(Duration::from_millis(2000));
    assert_eq!(cache3.get("key2"), None);
}

```


```
use thread_safe_cache::*;
fn main() {
        let mut builder: Builder<String,String> = Builder::init();
        builder.max_size(1000); // not implemented
        let cache_build = builder.build();
}

```

