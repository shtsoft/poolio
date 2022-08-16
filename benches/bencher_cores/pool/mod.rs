use std::panic::UnwindSafe;

pub trait Pool {
    fn new(number_of_workers: usize) -> Self;
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static;
    fn join(&self);
}

impl Pool for poolio::ThreadPool {
    fn new(number_of_workers: usize) -> Self {
        Self::new(number_of_workers, poolio::PanicSwitch::Kill).unwrap()
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static,
    {
        Self::execute(self, f);
    }

    fn join(&self) {}
}

impl Pool for threadpool::ThreadPool {
    fn new(number_of_workers: usize) -> Self {
        Self::new(number_of_workers)
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static,
    {
        Self::execute(self, f);
    }

    fn join(&self) {
        self.join();
    }
}
