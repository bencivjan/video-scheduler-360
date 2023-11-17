use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{channel, Receiver, Sender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};
// The timer we wrote in the previous section:
use crate::timer::TimerFuture;

/// Task executor that receives tasks off of a channel and runs them.
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
pub struct Spawner {
    task_sender: Sender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
pub struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: Sender<Arc<Task>>,
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (task_sender, ready_queue) = channel();
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    pub fn spawn(
        &self,
        future: impl Future<Output = Result<(), Box<dyn std::error::Error>>> + 'static + Send,
    ) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if future.as_mut().poll(context).is_pending() {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}

pub async fn gtyield(id: usize) {
    println!("Yielding thread {}", id);
    TimerFuture::new(Duration::new(0, 1)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let (executor, spawner) = new_executor_and_spawner();

        // Spawn a task to print before and after waiting on a timer.
        spawner.spawn(async {
            // let start = Instant::now();
            gtyield(1).await;
            println!("1");
            gtyield(1).await;
            // let end = Instant::now();
            // println!("done in {} seconds!", end.duration_since(start).as_secs());
            println!("5");
            Ok(())
        });

        spawner.spawn(async {
            println!("2");
            let mut a = 0;
            for _ in 0..100_000_000 {
                a += 1;
            }
            println!("3: Thread 2 counted to 100 mil");
            gtyield(2).await;
            println!("6");
            Ok(())
        });

        spawner.spawn(async {
            println!("4");
            gtyield(3).await;
            println!("7");
            Ok(())
        });

        // Drop the spawner so that our executor knows it is finished and won't
        // receive more incoming tasks to run.
        drop(spawner);

        // Run the executor until the task queue is empty.
        // This will print "howdy!", pause, and then print "done!".
        executor.run();
    }
}
