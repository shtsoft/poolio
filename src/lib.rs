//! poolio is a thread-pool-implementation using only channels for concurrency.
//!
//! ## Design
//!
//! A poolio-thread-pool is essentially made up by a 'supervisor'-thread and a specified number of 'worker'-threads.
//! A worker's only purpose is executing jobs (in the guise of closures) while the supervisor is responsible for anything else like - most importantly - assigning jobs to workers it gets from outside the pool via the public API.
//! To this end the thread-pool is set up in such a way that the supervisor can communicate with each worker seperately but concurrently.
//! This, in particular, ensures that each worker is equally busy.
//! A single supervisor-worker-communication is roughly as follows:
//! 1. worker tells its current status to the supervisor
//! 2. supervisor decides what to tell the worker to do on the basis of the current order-message from outside the pool and the worker-status
//! 3. supervisor tells the work what to do
//! 4. worker tries to do what it was told by the supervisor
//! 5. worker tells its current status to the supervisor
//!
//! The following graphic illustrates the aformentioned communication-model of a supervisor-thread S and a worker-thread W:
//!
//! <pre>
//!    W
//!    _
//!    .
//!    .
//!    send-status
//!    .   O
//!    .     O
//!    .       O                 send-message
//!    .         O                   O
//!    .           O               O
//!    recv         recv         O
//!   * .  O       O  . .      O
//!  .   .   O   O   .   .   O
//! .     e    O    m     recv . . | S
//!  .   .   O   O   .   *
//!   . .  O       O  . .
//!    send-status  send-message
//!
//! X | . . * : arrow starting at | and ending at * representing the control-flow of thread X
//! O O O O O : channel
//! e : execute job
//! m : manage workers
//! </pre>
//!
//! ## Usage
//!
//! To use a poolio-[`ThreadPool`] you simply have to set one up using the [`ThreadPool::new`]-method and task the pool to run jobs using the [`ThreadPool::execute`]-method.
//!
//! # Examples
//!
//! Setting up a pool to make some server multi-threaded:
//!
//! ```
//! fn handle(req: usize) {
//!     println!("Handled!")
//! }
//!
//! let server_requests = [1, 2, 3, 4, 5, 6, 7, 8, 9];
//!
//! let pool = poolio::ThreadPool::new(3, poolio::PanicSwitch::Kill).unwrap();
//!
//! for req in server_requests {
//!     pool.execute(move || {
//!         handle(req);
//!     });
//! }
//! ```

use std::fmt;
use std::panic::UnwindSafe;
use std::sync::mpsc::{channel, Sender};
use std::thread;

//TODO put ThreadHandle into a module (don't forget to include the unit tests)
/// Wraps [`std::thread::JoinHandle<T>`] to work around ownership/borrowing-issues of threads embedded in a `struct`.
type ThreadHandle = Option<thread::JoinHandle<()>>;

/// Joins a thread wrapped in a [`ThreadHandle`].
/// - `thread` is the handle obtained with the help of [`std::option::Option<T>::take`] mitigating borrow-complaints at the call site.
///
/// # Panics
///
/// A panic is caused if the `thread` is `None` or if joining the thread fails (which is only the case when the thread has panicked).
fn join(thread: ThreadHandle) {
    match thread {
        Some(thread) => {
            if let Err(e) = thread.join() {
                panic!("{:?}", e);
            }
        }
        None => panic!("Cannot join: no thread has been provided."),
    };
}

/// Types the jobs the [`ThreadPool`] can run.
type Job = Box<dyn FnOnce() + UnwindSafe + Send + 'static>;

/// Defines what the [`ThreadPool`] can be ordered to do.
enum Message {
    /// Order the pool to execute a job.
    NewJob(Job),
    /// Order the pool to finish its remaining jobs and shut down afterwards.
    Terminate,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NewJob(_) => write!(f, "[NewJob]"),
            Self::Terminate => write!(f, "[Terminate]"),
        }
    }
}

/// Configures what the [`ThreadPool`] is supposed to do in case of a 'panicking job', that is, a job which panics while running in a thread.
pub enum PanicSwitch {
    /// Configure the pool to finish parallely running jobs and then kill the whole process in case of a panicked job.
    Kill,
    /// Configure the pool to ignore panicked jobs and just respawn the polluted threads.
    Respawn,
}

/// Abstracts the thread-pools.
pub struct ThreadPool {
    /// interface to the pool-controlling thread
    supervisor: Supervisor,
}

impl ThreadPool {
    /// Sets up a new pool.
    /// - `size` is the (non-zero) number of worker-threads in the pool.
    /// - `mode` is the setting of the panic switch.
    ///
    /// # Errors
    ///
    /// An error is returned if 0 was passed as `size` (since a pool without worker-threads does not make sense).
    ///
    /// # Examples
    ///
    /// Setting up a pool with three worker-threads in kill-mode:
    ///
    /// ```
    /// let pool = poolio::ThreadPool::new(3, poolio::PanicSwitch::Kill).unwrap();
    /// ```
    pub fn new<'a>(size: usize, mode: PanicSwitch) -> Result<Self, &'a str> {
        if size == 0 {
            return Err("Setting up a pool with no workers is not allowed.");
        };

        let pool = Self {
            supervisor: Supervisor::new(size, mode),
        };
        Ok(pool)
    }

    /// Runs a job in `self`.
    /// - `f` is the job to be run and has to be provided as a certain closure.
    ///
    /// Note that if `f` panics the behavior is according to the setting of the [`PanicSwitch`] of `self`.
    ///
    /// # Panics
    ///
    /// A panic is caused if the pool is unreachable.
    ///
    /// # Examples
    ///
    /// Setting up a pool and printing two strings concurrently:
    ///
    /// ```
    /// let pool = poolio::ThreadPool::new(2, poolio::PanicSwitch::Kill).unwrap();
    /// pool.execute(|| println!{"house"});
    /// pool.execute(|| println!{"cat"});
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + UnwindSafe + Send + 'static,
    {
        let job = Box::new(f);

        self.send(Message::NewJob(job));
    }

    /// Tries to shut down `self` gracefully.
    ///
    /// In particular, one has to assume that all remaining jobs will be finished (modulo panics in [`PanicSwitch::Kill`]-mode).
    ///
    /// # Panics
    ///
    /// A panic occurs if
    /// 1. the pool is unreachable.
    /// 2. joining the threads panics.
    fn terminate(&mut self) {
        self.send(Message::Terminate);

        join(self.supervisor.thread.take());
    }

    /// Wraps sending a [`Message`] to the pool.
    ///
    /// # Panics
    ///
    /// A panic is caused if the receiver has already been deallocated.
    fn send(&self, msg: Message) {
        let panic_message = format!("Ordering {} failed. Pool is unreachable.", msg);

        self.supervisor.orders_s.send(msg).expect(&panic_message);
    }
}

impl Drop for ThreadPool {
    /// Tries to shut down `self` gracefully.
    ///
    /// In particular, one has to assume that all remaining jobs will be finished (modulo panics in [`PanicSwitch::Kill`] setting).
    ///
    /// # Panics
    ///
    /// A panic occurs if
    /// 1. the the pool is unreachable
    /// 2. joining the threads panics.
    ///
    /// Remember that a panic while dropping aborts the whole process.
    fn drop(&mut self) {
        self.terminate();
    }
}

/// [`StaffNumber`]s identify workers.
type StaffNumber = usize;

/// [`Status`] is what worker with [`StaffNumber`] is currently doing.
enum Status {
    /// worker `id` is idle.
    Idle(StaffNumber),
    /// worker `id` has a panicked job.
    Panic(StaffNumber),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Idle(_) => write!(f, "[idle]"),
            Status::Panic(_) => write!(f, "[panic]"),
        }
    }
}

/// [`Supervisor`] abstracts the supervisors.
struct Supervisor {
    /// place to put orders
    orders_s: Sender<Message>,
    /// handle to join
    thread: ThreadHandle,
}

impl Supervisor {
    /// Sets up a supervisor.
    /// - `number_of_workers` is how many workers are employed.
    /// - `mode` configures what happens when workers report panicking jobs.
    ///
    /// In particular, it spawns a thread and sets up a way to communicate to the thread.
    /// Moreover it creates the workers controlled by the just spawned supervisor-thread.
    fn new(mut number_of_workers: usize, mode: PanicSwitch) -> Self {
        // this channel is used by the pool to contact the supervisor
        let (orders_s, orders_r) = channel();

        let thread = thread::spawn(move || {
            // this channel is used by the workers to contact the supervisor
            let (statuses_s, statuses_r) = channel();

            // consrtruct `number_of_workers` worker-threads
            let mut workers = Vec::with_capacity(number_of_workers);
            for id in 0..number_of_workers {
                workers.push(Worker::new(id, statuses_s.clone()));
            }

            // track how many jobs have panicked
            let mut panicked_jobs = 0;

            // keepin' running to distribute jobs among idle workers
            'distribute_jobs: while let Message::NewJob(job) = orders_r.recv().unwrap() {
                'query_status: loop {
                    match statuses_r.recv().unwrap() {
                        Status::Idle(id) => {
                            workers[id]
                                .instructions_s
                                .send(Message::NewJob(job))
                                .unwrap();
                            break 'query_status;
                        }
                        Status::Panic(id) => {
                            join(workers[id].thread.take());
                            match mode {
                                PanicSwitch::Kill => {
                                    panicked_jobs += 1;
                                    number_of_workers -= 1;
                                    break 'distribute_jobs;
                                }
                                PanicSwitch::Respawn => {
                                    workers[id] = Worker::new(id, statuses_s.clone());
                                }
                            };
                        }
                    }
                }
            }

            // destruct all remaining worker-threads
            while number_of_workers != 0 {
                match statuses_r.recv().unwrap() {
                    Status::Idle(id) => {
                        workers[id].instructions_s.send(Message::Terminate).unwrap();
                        join(workers[id].thread.take());
                    }
                    Status::Panic(id) => {
                        join(workers[id].thread.take());
                        if let PanicSwitch::Kill = mode {
                            panicked_jobs += 1;
                        };
                    }
                };
                number_of_workers -= 1;
            }

            if panicked_jobs > 0 {
                eprintln!("Aborting process: {} panicked jobs.", panicked_jobs);
                std::process::abort();
            }

            // ensure that `orders_r` lives as long as the thread to prevent reachability-errors
            drop(orders_r);
        });

        Self {
            orders_s,
            thread: Some(thread),
        }
    }
}

/// [`Worker`] abstracts workers.
struct Worker {
    /// place to put instructions
    instructions_s: Sender<Message>,
    /// handle to join
    thread: ThreadHandle,
}

impl Worker {
    /// Sets up a new worker.
    /// - `id` is the worker's staff number.
    /// - `statuses_s` is where the worker puts its current status.
    ///
    /// In particular, it spawns a thread and sets up a way to communicate to the thread.
    fn new(id: StaffNumber, statuses_s: Sender<Status>) -> Self {
        // this channel is used by the supervisor to contact this worker
        let (instructions_s, instructions_r) = channel();

        let thread = thread::spawn(move || {
            // report for duty
            statuses_s.send(Status::Idle(id)).unwrap();

            // keepin' running to execute jobs
            loop {
                let message = instructions_r.recv().unwrap();

                match message {
                    Message::NewJob(job) => match std::panic::catch_unwind(job) {
                        Ok(_) => {
                            statuses_s.send(Status::Idle(id)).unwrap();
                        }
                        Err(_) => {
                            statuses_s.send(Status::Panic(id)).unwrap();
                            break;
                        }
                    },
                    Message::Terminate => break,
                }
            }
        });

        Self {
            instructions_s,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    // settings
    const SIZE: usize = 2; //= 6; && = 12; && = 36;
    const MODE: PanicSwitch = PanicSwitch::Respawn; //= PanicSwitch::Kill;
    const ID: StaffNumber = 0;

    #[test]
    fn test_threadhandle_join() {
        join(Some(thread::spawn(|| {})));
    }

    #[test]
    #[should_panic]
    fn test_threadhandle_join_panic_some() {
        join(Some(thread::spawn(|| panic!("Oh no!"))));
    }

    #[test]
    #[should_panic]
    fn test_threadhandle_join_panic_none() {
        join(None);
    }

    #[test]
    fn test_threadpool_new_ok() {
        let pool = ThreadPool::new(SIZE, MODE);
        assert!(matches!(pool, Ok(_)));
    }

    #[test]
    fn test_threadpool_new_err() {
        let pool = ThreadPool::new(0, MODE);
        assert!(matches!(pool, Err(_)));
    }

    #[test]
    fn test_threadpool_execute() {
        const N: usize = 5;

        let pool = ThreadPool::new(SIZE, MODE).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));

        let count_to = |n: usize| {
            for _ in 0..n {
                let counter = Arc::clone(&counter);
                pool.execute(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                });
            }
        };

        for _ in 0..N {
            count_to(SIZE);
            if let PanicSwitch::Respawn = MODE {
                pool.execute(|| panic!("Oh no!"));
            }
        }

        drop(pool);

        assert_eq!(N * SIZE, counter.load(Ordering::SeqCst));
    }

    #[test]
    fn test_worker_thread_newjob() {
        let (statuses_s, statuses_r) = channel();
        let worker = Worker::new(ID, statuses_s);

        assert!(matches!(statuses_r.recv().unwrap(), Status::Idle(ID)));

        let flag = Arc::new(AtomicBool::new(false));
        let flag_ref = Arc::clone(&flag);
        let job = Box::new(move || {
            flag_ref.store(true, Ordering::SeqCst);
        });
        worker.instructions_s.send(Message::NewJob(job)).unwrap();
        assert!(matches!(statuses_r.recv().unwrap(), Status::Idle(ID)));
        assert!(flag.load(Ordering::SeqCst));

        let job = Box::new(|| panic!("Oh no!"));
        worker.instructions_s.send(Message::NewJob(job)).unwrap();
        assert!(matches!(statuses_r.recv().unwrap(), Status::Panic(ID)));

        join(worker.thread);
    }

    #[test]
    fn test_worker_thread_terminate() {
        let (statuses_s, statuses_r) = channel();
        let worker = Worker::new(ID, statuses_s);

        assert!(matches!(statuses_r.recv().unwrap(), Status::Idle(ID)));

        worker.instructions_s.send(Message::Terminate).unwrap();

        join(worker.thread);
    }
}
