use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::Lazy;

use super::utils::abort_on_panic;
use super::{JoinKarx, Karx};

/// Spawns a blocking Karx.
///
/// The Karx will be spawned onto a thread pool specifically dedicated to blocking Karx. This
/// is useful to prevent long-running synchronous operations from blocking the main futures
/// executor.
///
/// See also: [`karx::exec`], [`karx::spawn`].
///
/// [`karx::exec`]: fn.exec.html
/// [`karx::spawn`]: fn.spawn.html
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # kayrx::karx::exec(async {
/// #
/// use kayrx::karx;
///
/// karx::block(|| {
///     println!("long-running task here");
/// })
/// .await;
/// #
/// # })
/// ```
#[inline]
pub fn block<F, T>(f: F) -> JoinKarx<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let schedule = |task| POOL.sender.send(task).unwrap();
    let (task, handle) = super::kernel::spawn(async { f() }, schedule, Karx::new(None));
    task.schedule();
    JoinKarx::new(handle)
}

type Runnable = super::kernel::Task<Karx>;

struct Pool {
    sender: Sender<Runnable>,
    receiver: Receiver<Runnable>,
}

/// The number of sleeping worker threads.
static SLEEPING: AtomicUsize = AtomicUsize::new(0);

static POOL: Lazy<Pool> = Lazy::new(|| {
    // Start a single worker thread waiting for the first Karx.
    start_thread();

    let (sender, receiver) = unbounded();
    Pool { sender, receiver }
});

fn start_thread() {
    SLEEPING.fetch_add(1, Ordering::SeqCst);
    let timeout = Duration::from_secs(1);

    thread::Builder::new()
        .name("karx/blocking".to_string())
        .spawn(move || {
            loop {
                let mut task = match POOL.receiver.recv_timeout(timeout) {
                    Ok(task) => task,
                    Err(_) => {
                        // Check whether this is the last sleeping thread.
                        if SLEEPING.fetch_sub(1, Ordering::SeqCst) == 1 {
                            // If so, then restart the thread to make sure there is always at least
                            // one sleeping thread.
                            if SLEEPING.compare_and_swap(0, 1, Ordering::SeqCst) == 0 {
                                continue;
                            }
                        }

                        // Stop the thread.
                        return;
                    }
                };

                // If there are no sleeping threads, then start one to make sure there is always at
                // least one sleeping thread.
                if SLEEPING.fetch_sub(1, Ordering::SeqCst) == 1 {
                    start_thread();
                }

                loop {
                    // Run the Karx.
                    abort_on_panic(|| task.run());

                    // Try taking another Karx if there are any available.
                    task = match POOL.receiver.try_recv() {
                        Ok(task) => task,
                        Err(_) => break,
                    };
                }

                // If there is at least one sleeping thread, stop this thread instead of putting it
                // to sleep.
                if SLEEPING.load(Ordering::SeqCst) > 0 {
                    return;
                }

                SLEEPING.fetch_add(1, Ordering::SeqCst);
            }
        })
        .expect("cannot start a blocking thread");
}
