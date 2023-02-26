use std::thread;
use std::thread::sleep;
use std::time::Duration;
use super::*;

#[test]
fn common() {
    {
        let cache_init: ThreadSafeCache<String, i32> = ThreadSafeCache::new();

        let mut cache1 = cache_init.clone();
        thread::spawn(move || {
            // println!("thread 1");
            cache1.put("a".to_string(), 1);
            cache1.put_exp("b".to_string(), 2, 1000);
            // println!("thread 1.");
        });
        let mut cache2 = cache_init.clone();
        let t = thread::spawn(move || {
            // println!("thread 2");
            sleep(Duration::from_millis(100));
            let ret = cache2.get("a".to_string());
            // println!("thread 2.");
            ret
        });
        assert_eq!(t.join().unwrap(), Some(1));

        thread::sleep(Duration::from_millis(100));
        let mut cache3 = cache_init.clone();
        assert_eq!(cache3.get("b".to_string()), Some(2));
        thread::sleep(Duration::from_millis(3000));
        assert_eq!(cache3.get("b".to_string()), None);
    }

}


#[test]
fn builder_max_size() {
    let mut builder: Builder<String, i32> = Builder::init();
    builder.max_size(2);
    let mut cache = builder.build();
    cache.put("a".to_string(), 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("b".to_string(), 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("c".to_string(), 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("d".to_string(), 1);
    assert_eq!(cache.get("a".to_string()), None);
    assert_eq!(cache.get("b".to_string()), None);
    assert_eq!(cache.get("c".to_string()), Some(1));
    assert_eq!(cache.get("d".to_string()), Some(1));

}

#[test]
fn save_test() {
    let mut builder: Builder<String, i32> = Builder::init();
    builder.max_size(1000);
    let mut cache = builder.build();
    cache.put("a".to_string(), 1);
    cache.save("test.db");

    let mut builder: Builder<String, i32> = Builder::init();
    builder.max_size(1000);
    let mut cache_clean = builder.build();
    cache_clean.load("test.db");
    assert_eq!(cache_clean.get("a".to_string()), Some(1));


}
