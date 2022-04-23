use poolio::{PanicSwitch, ThreadPool};

#[test]
#[ignore]
fn test_load() {
    const SIZE: usize = 5;
    const N: u32 = 5;

    let pool_kill = ThreadPool::new(SIZE, PanicSwitch::Kill).unwrap();
    let pool_respawn = ThreadPool::new(SIZE, PanicSwitch::Respawn).unwrap();

    for n in 0..(N as usize) * SIZE {
        let job = move || {
            for i in 0..n.pow(N) {
                println!("{}", i);
            }
        };

        pool_kill.execute(job);

        pool_respawn.execute(|| panic!("Oh no!"));

        pool_respawn.execute(job);
    }
}
