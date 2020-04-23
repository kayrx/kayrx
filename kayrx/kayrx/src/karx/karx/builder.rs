use std::future::Future;
use std::io;

use super::utils::abort_on_panic;
use super::{JoinKarx, Karx};
use crate::karx::RUNTIME;

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

        let future = async move {
            // Drop Karx-locals on exit.
            defer! {
                Karx::get_current(|t| unsafe { t.drop_locals() });
            }

            future.await
        };

        let schedule = move |t| RUNTIME.schedule(Runnable(t));
        let (task, handle) = super::kernel::spawn(future, schedule, task);
        task.schedule();
        Ok(JoinKarx::new(handle))
    }
}

/// A runnable Karx.
pub struct Runnable(super::kernel::Task<Karx>);

impl Runnable {
    /// Runs the task by polling its future once.
    pub fn run(self) {
        unsafe {
            Karx::set_current(self.0.tag(), || abort_on_panic(|| self.0.run()));
        }
    }
}
