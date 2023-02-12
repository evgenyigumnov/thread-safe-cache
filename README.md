# thread-safe-cache

```
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

```

