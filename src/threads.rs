//  ThreadPool structure from tutorial: 
//  https://doc.rust-lang.org/book/ch20-02-multithreaded.html 

use std::{
    sync::{mpsc, Arc, Mutex},
    thread
};


// Main structure in this file
// Requires size to be given when created
pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

// It is a smart pointer to function that:
// - will be executed only once
// - will be sent through threads
// - has lifetime of the program
type Job = Box<dyn FnOnce() + Send + 'static>

impl ThreadPool {
    // Create wanted amount of workers and prepare them
    // Takes:
    // size - amount of workers to be created
    //
    // Returns:
    // ThreadPool instance
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // Smart pointer to Mutex object
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        // Creating workers
        for id in 0..size {
            // Clone the pointer receiver, because each worker needs access to it
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool{ workers, sender }
    }

    // Runs given function by passing it to workers
    // Takes:
    // f - function to be ran

    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // Pass function(job) to workers
        // unwrap in case of an error
        self.sender.send(job).unwrap();
    }
}

// Waiting for new jobs to appear
// Amount of workers is declared as size argument for ThreadPool
pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Take a function(job) to execute
            // Only there are jobs to be done
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            // Execute function given to ThreadPool
            job();
        });

        Woker{ id, thread }
    }
}