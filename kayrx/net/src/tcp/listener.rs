use async_ready::AsyncReady;
use futures_core::stream::Stream;
use futures_util::ready;
use std::fmt;
use std::io;
use std::net::{self, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};

use super::TcpStream;
use crate::driver::lxar;
use crate::driver::PollEvented;

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    io: PollEvented<lxar::net::TcpListener>,
}

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<TcpListener> {
        let l = lxar::net::TcpListener::bind(addr)?;
        Ok(TcpListener::new(l))
    }

    fn new(listener: lxar::net::TcpListener) -> TcpListener {
        let io = PollEvented::new(listener);
        TcpListener { io }
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    pub fn incoming(&mut self) -> Incoming<'_> {
        Incoming { inner: self }
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.io.get_ref().ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.io.get_ref().set_ttl(ttl)
    }

    fn poll_accept_std(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(net::TcpStream, SocketAddr)>> {
        ready!(Pin::new(&mut self.io).poll_read_ready(cx)?);

        match Pin::new(&mut self.io).get_ref().accept_std() {
            Ok(pair) => Poll::Ready(Ok(pair)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Pin::new(&mut self.io).clear_read_ready(cx)?;
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncReady for TcpListener {
    type Ok = (TcpStream, SocketAddr);
    type Err = std::io::Error;

    /// Check if the stream can be read from.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Self::Ok, Self::Err>> {
        let (io, addr) = ready!(self.poll_accept_std(cx)?);
        let io = lxar::net::TcpStream::from_stream(io)?;
        let io = TcpStream::new(io);
        Poll::Ready(Ok((io, addr)))
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

mod sys {
    use super::TcpListener;
    use std::os::unix::prelude::*;

    impl AsRawFd for TcpListener {
        fn as_raw_fd(&self) -> RawFd {
            self.io.get_ref().as_raw_fd()
        }
    }
}

/// Stream returned by the `TcpListener::incoming` function representing the
/// stream of sockets received from a listener.
#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct Incoming<'a> {
    inner: &'a mut TcpListener,
}

impl<'a> Stream for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(Pin::new(&mut *self.inner).poll_ready(cx)?);
        Poll::Ready(Some(Ok(socket)))
    }
}
