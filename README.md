# thread-safe-cache

## Features

- Thread safe
- Key/Value cache
- Expiration
- Save/Load to file
- Embedded mode
- Max size

## Todo

- Client/Server mode
- LRU
- Tags


## Usage


```
use thread_safe_cache::*;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
fn main() {
    let mut builder: BuilderEmbedded<String, i32> = BuilderEmbedded::init();
    builder.max_size(1000);
    let mut cache_init = builder.build();
    let mut cache1 = cache_init.clone();
    thread::spawn(move || {
        cache1.put("key1".to_string(), 1);
        cache1.put_exp("key2".to_string(), 2, 3000);
    });
    let mut cache2 = cache_init.clone();
    let t = thread::spawn(move || {
        sleep(Duration::from_millis(2000));
        cache2.rm("key1".to_string());
        cache2.get("key2".to_string())
    });
    assert_eq!(t.join().unwrap(),Some(2));
    let mut cache3 = cache_init.clone();
    assert_eq!(cache3.get("key2".to_string()), Some(2));
    sleep(Duration::from_millis(2000));
    assert_eq!(cache3.get("key2".to_string()), None);
}

```


```
use thread_safe_cache::*;
fn main() {
    let mut builder: BuilderEmbedded<String, i32> = BuilderEmbedded::init();
    let mut cache = builder.build();
    cache.put("a".to_string(), 1);
    cache.save("test.db");

    let mut builder: Builder<String, i32> = Builder::init();
    builder.max_size(1000);
    let mut cache_clean = builder.build();
    cache_clean.load("test.db");
    assert_eq!(cache_clean.get("a".to_string()), Some(1));
}

```

