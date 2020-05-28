use libc::{self, c_int};

#[macro_use]
pub mod dlsym;

mod awakener;
mod epoll;
mod io;
mod ready;
mod tcp;
mod udp;

pub use self::awakener::Awakener;
pub use self::epoll::{Events, Selector};
pub use self::io::{set_nonblock, Io};
pub use self::ready::{UnixReady, READY_ALL};
pub use self::tcp::{TcpListener, TcpStream};
pub use self::udp::UdpSocket;

pub use iovec::IoVec;
use std::os::unix::io::FromRawFd;

pub fn pipe() -> std::io::Result<(Io, Io)> {
    // Use pipe2 for atomically setting O_CLOEXEC if we can, but otherwise
    // just fall back to using `pipe`.
    dlsym!(fn pipe2(*mut c_int, c_int) -> c_int);

    let mut pipes = [0; 2];
    unsafe {
        match pipe2.get() {
            Some(pipe2_fn) => {
                let flags = libc::O_NONBLOCK | libc::O_CLOEXEC;
                cvt(pipe2_fn(pipes.as_mut_ptr(), flags))?;
                Ok((Io::from_raw_fd(pipes[0]), Io::from_raw_fd(pipes[1])))
            }
            None => {
                cvt(libc::pipe(pipes.as_mut_ptr()))?;
                // Ensure the pipe are closed if any of the system calls below
                // fail.
                let r = Io::from_raw_fd(pipes[0]);
                let w = Io::from_raw_fd(pipes[1]);
                cvt(libc::fcntl(pipes[0], libc::F_SETFD, libc::FD_CLOEXEC))?;
                cvt(libc::fcntl(pipes[1], libc::F_SETFD, libc::FD_CLOEXEC))?;
                cvt(libc::fcntl(pipes[0], libc::F_SETFL, libc::O_NONBLOCK))?;
                cvt(libc::fcntl(pipes[1], libc::F_SETFL, libc::O_NONBLOCK))?;
                Ok((r, w))
            }
        }
    }
}

trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

impl IsMinusOne for i32 {
    fn is_minus_one(&self) -> bool {
        *self == -1
    }
}
impl IsMinusOne for isize {
    fn is_minus_one(&self) -> bool {
        *self == -1
    }
}

fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    use std::io;

    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
