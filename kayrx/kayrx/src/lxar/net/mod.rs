//! Networking primitives
//!
//! The types provided in this module are non-blocking by default and are
//! designed to for Linux.

mod tcp;
mod udp;
mod uds;

pub use self::tcp::{TcpListener, TcpStream};
pub use self::udp::UdpSocket;
pub use self::uds::datagram::UnixDatagram;
pub use self::uds::listener::UnixListener;
pub use self::uds::stream::UnixStream;
