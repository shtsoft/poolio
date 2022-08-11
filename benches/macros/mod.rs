#[macro_export]
macro_rules! bench_identifier {
    ($function:expr, $cr8:literal, $size:expr) => {
        &format!("{}:{}-pool-({}-worker(s))", $function, $cr8, $size)
    };
}

#[macro_export]
macro_rules! bencher {
    ($function:expr, $Pool:ty, $size:expr) => {
        |b| {
            b.iter(|| {
                let _: Option<$Pool> = $function(black_box($size));
            })
        }
    };
}
