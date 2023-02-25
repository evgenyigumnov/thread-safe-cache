use std::thread;
use std::thread::sleep;
use std::time::Duration;
use super::*;

#[test]
fn common() {
    {
        let cache_init: ThreadSafeCache<&str, i32> = ThreadSafeCache::new();

        let mut cache1 = cache_init.clone();
        thread::spawn(move || {
            // println!("thread 1");
            cache1.put("a", 1);
            cache1.put_exp("b", 2, 1000);
            // println!("thread 1.");
        });
        let mut cache2 = cache_init.clone();
        let t = thread::spawn(move || {
            // println!("thread 2");
            sleep(Duration::from_millis(100));
            let ret = cache2.get("a");
            // println!("thread 2.");
            ret
        });
        assert_eq!(t.join().unwrap(), Some(1));

        thread::sleep(Duration::from_millis(100));
        let mut cache3 = cache_init.clone();
        assert_eq!(cache3.get("b"), Some(2));
        thread::sleep(Duration::from_millis(3000));
        assert_eq!(cache3.get("b"), None);
    }

}


#[test]
fn builder_max_size() {
    let mut builder: Builder<&str, i32> = Builder::init();
    builder.max_size(2);
    let mut cache = builder.build();
    cache.put("a", 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("b", 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("c", 1);
    thread::sleep(Duration::from_millis(100));
    cache.put("d", 1);
    assert_eq!(cache.get("a"), None);
    assert_eq!(cache.get("b"), None);
    assert_eq!(cache.get("c"), Some(1));
    assert_eq!(cache.get("d"), Some(1));

}
