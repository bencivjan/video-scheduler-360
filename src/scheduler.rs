use std::{
    sync::{mpsc::{sync_channel, Receiver, SyncSender}, Arc, Mutex},
    thread,
};

pub struct Scheduler {
    // queue of GreenThreads
    ready_queue: Receiver<Arc<GreenThread>>
}

type Job = Box<dyn Fn() + Send + 'static>;

impl Scheduler {
    fn run(&self) {
        while let Ok(gt) = self.ready_queue.recv() {
            println!("Received thread {}", gt.id);
        }
    }
}

pub struct GreenThread {
    // Thread id
    id: usize,
    // The code to execute
    f: Box<dyn Fn()>
    // The state (wait?, ready, running)
}

impl GreenThread {
    fn new() {

    }

    fn gtyield() {
        
    }
}

fn new_scheduler_and_spawner() -> (Executor, Spawner) {
    // Maximum number of tasks to allow queueing in the channel at once.
    // This is just to make `sync_channel` happy, and wouldn't be present in
    // a real executor.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

#[cfg(test)]
mod tests {
    use crate::scheduler::*;

    #[test]
    fn test_create_gt() {
        
    }
}