use poolio::{PanicSwitch, ThreadPool};
use std::sync::{Arc, Mutex};

#[test]
#[allow(clippy::mutex_atomic)]
fn test_basic() {
    const SIZE: usize = 2;
    const MODE: PanicSwitch = PanicSwitch::Kill; //= PanicSwitch::Respawn;

    let pool = ThreadPool::new(SIZE, MODE).unwrap();

    let counter = Arc::new(Mutex::new(0));

    for _ in 0..SIZE {
        let counter = Arc::clone(&counter);
        pool.execute(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
    }

    drop(pool);

    assert_eq!(SIZE, *counter.lock().unwrap());
}
