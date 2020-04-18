use std::future::Future;
use std::io;

use kv_log_macro::trace;

use crate::karx::RUNTIME;
use crate::karx::{JoinKarx, Karx};
use crate::karx::utils::abort_on_panic;

/// Karx builder that configures the settings of a new Karx.
#[derive(Debug, Default)]
pub struct Builder {
    pub(crate) name: Option<String>,
}

impl Builder {
    /// Creates a new builder.
    #[inline]
    pub fn new() -> Builder {
        Builder { name: None }
    }

    /// Configures the name of the Karx.
    #[inline]
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// Spawns a Karx with the configured settings.
    pub fn spawn<F, T>(self, future: F) -> io::Result<JoinKarx<T>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Create a new Karx handle.
        let task = Karx::new(self.name);

        // Log this `spawn` operation.
        trace!("spawn", {
            task_id: task.id().0,
            parent_task_id: Karx::get_current(|t| t.id().0).unwrap_or(0),
        });

        let future = async move {
            // Drop Karx-locals on exit.
            defer! {
                Karx::get_current(|t| unsafe { t.drop_locals() });
            }

            // Log completion on exit.
            defer! {
                trace!("completed", {
                    task_id: Karx::get_current(|t| t.id().0),
                });
            }

            future.await
        };

        let schedule = move |t| RUNTIME.schedule(Runnable(t));
        let (task, handle) = crate::karx::kernel::spawn(future, schedule, task);
        task.schedule();
        Ok(JoinKarx::new(handle))
    }
}

/// A runnable Karx.
pub struct Runnable(crate::karx::kernel::Task<Karx>);

impl Runnable {
    /// Runs the task by polling its future once.
    pub fn run(self) {
        unsafe {
            Karx::set_current(self.0.tag(), || abort_on_panic(|| self.0.run()));
        }
    }
}
