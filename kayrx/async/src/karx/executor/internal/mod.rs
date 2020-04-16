// extra doc:
// Inspired by golang runtime, see https://golang.org/src/runtime/proc.go
// so understand some terminology like machine and processor will help you
// understand this code.

mod machine;
mod processor;
mod system;
mod task;

use std::future::Future;
use crate::karx;

use self::system::SYSTEM;
use self::task::TaskTag;

type Task = karx::Task<TaskTag>;

/// Run the task.
pub fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
  let (task, _) = karx::spawn(f, |t| SYSTEM.push(t), TaskTag::new());
  task.schedule();
}
