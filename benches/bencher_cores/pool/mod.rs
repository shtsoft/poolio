use std::panic::UnwindSafe;

pub trait Pool {
    fn new(number_of_workers: usize) -> Self;
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static;
}

impl Pool for poolio::ThreadPool {
    fn new(number_of_workers: usize) -> poolio::ThreadPool {
        poolio::ThreadPool::new(number_of_workers, poolio::PanicSwitch::Kill).unwrap()
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static,
    {
        poolio::ThreadPool::execute(self, f);
    }
}

impl Pool for rayon::ThreadPool {
    fn new(number_of_workers: usize) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_workers)
            .build()
            .unwrap()
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static,
    {
        rayon::ThreadPool::install(self, f);
    }
}
