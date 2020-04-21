use async_ready::{AsyncReady, TakeError};
use futures_util::ready;
use futures_core::Stream;
use std::fmt;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::{self, SocketAddr};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::UnixStream;
use crate::lxar;
use crate::net::reactor::PollEvented;

/// A Unix socket cna accept connections from other Unix sockets.
pub struct UnixListener {
    io: PollEvented<lxar::net::UnixListener>,
}

impl UnixListener {

    pub fn bind(path: impl AsRef<Path>) -> io::Result<UnixListener> {
        let listener = lxar::net::UnixListener::bind(path)?;
        let io = PollEvented::new(listener);
        Ok(UnixListener { io })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    pub fn incoming(self) -> Incoming {
        Incoming::new(self)
    }

    fn poll_accept_std(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(net::UnixStream, SocketAddr)>> {
        ready!(Pin::new(&mut self.io).poll_read_ready(cx)?);

        match Pin::new(&mut self.io).get_ref().accept_std() {
            Ok(Some((sock, addr))) => Poll::Ready(Ok((sock, addr))),
            Ok(None) => {
                Pin::new(&mut self.io).clear_read_ready(cx)?;
                Poll::Pending
            }
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                Pin::new(&mut self.io).clear_read_ready(cx)?;
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl AsyncReady for UnixListener {
    type Ok = (UnixStream, SocketAddr);
    type Err = std::io::Error;

    /// Check if the stream can be read from.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Self::Ok, Self::Err>> {
        let (io, addr) = ready!(self.poll_accept_std(cx)?);
        let io = lxar::net::UnixStream::from_stream(io)?;
        Poll::Ready(Ok((UnixStream::new(io), addr)))
    }
}

impl TakeError for UnixListener {
    type Ok = io::Error;
    type Err = io::Error;

    fn take_error(&self) -> Result<Option<Self::Ok>, Self::Err> {
        self.io.get_ref().take_error()
    }
}

impl fmt::Debug for UnixListener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

impl AsRawFd for UnixListener {
    fn as_raw_fd(&self) -> RawFd {
        self.io.get_ref().as_raw_fd()
    }
}

/// Stream of listeners
#[derive(Debug)]
pub struct Incoming {
    inner: UnixListener,
}

impl Incoming {
    pub(crate) fn new(listener: UnixListener) -> Incoming {
        Incoming { inner: listener }
    }
}

impl Stream for Incoming {
    type Item = io::Result<UnixStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(Pin::new(&mut self.inner).poll_ready(cx)?);
        Poll::Ready(Some(Ok(socket)))
    }
}
