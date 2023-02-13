# thread-safe-cache

```
use thread_safe_cache::*;

let cache_init: ThreadSafeCache<&str, i32> = ThreadSafeCache::new();
let mut cache1 = cache_init.clone();
thread::spawn(move || {
  cache1.put("a", 1);
  cache1.put_exp("b", 2, 1000);
});
let mut cache2 = cache_init.clone();
let t = thread::spawn(move || {
  sleep(Duration::from_millis(2000));
  cache2.rm("a");
  cache2.get("b")
});
assert_eq!(t.join().unwrap(),None);

```

