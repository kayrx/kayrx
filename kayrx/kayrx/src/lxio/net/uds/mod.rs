//! MIO bindings for Unix Domain Sockets

#![deny(missing_docs)]

use std::io;

pub mod datagram;
pub mod listener;
mod socket;
pub mod stream;

fn cvt(i: libc::c_int) -> io::Result<libc::c_int> {
    if i == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(i)
    }
}