//! Networking primitives
//!
//! The types provided in this module are non-blocking by default and are
//! designed to be portable across all supported Mio platforms. As long as the
//! [portability guidelines] are followed, the behavior should be identical no
//! matter the target platform.
//!
//! [portability guidelines]: ../struct.Poll.html#portability

mod tcp;
mod udp;
pub mod uds;

pub use self::tcp::{TcpListener, TcpStream};
pub use self::udp::UdpSocket;
pub use self::uds::stream::UnixStream;
pub use self::uds::listener::UnixListener;
pub use self::uds::datagram::UnixDatagram;
