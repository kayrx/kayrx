//! # Async network TCP, UDP, UDS
//!
//! The types  are designed to closely follow the APIs of the
//! analogous types in `std::net` in `Asychronous` versions.
//!
//! # Examples
//! __TCP Server__
//! ```rust
//! use kayrx::net::{TcpListener, TcpStream};
//! use futures::prelude::*;
//!
//! async fn say_hello(mut stream: TcpStream) {
//!     stream.write_all(b"Shall I hear more, or shall I speak at this?").await;
//! }
//!
//! async fn listen() -> Result<(), Box<dyn std::error::Error + 'static>> {
//!     let socket_addr = "127.0.0.1:8080".parse()?;
//!     let mut listener = TcpListener::bind(&socket_addr)?;
//!     let mut incoming = listener.incoming();
//!
//!     // accept connections and process them serially
//!     while let Some(stream) = incoming.next().await {
//!         say_hello(stream?).await;
//!     }
//!     Ok(())
//! }
//! ```
//! __TCP Client__
//! ```rust,no_run
//! use std::error::Error;
//! use futures::prelude::*;
//! use kayrx::net::{TcpListener, TcpStream};
//!
//! async fn receive_sonnet() -> Result<(), Box<dyn Error + 'static>> {
//!     let socket_addr = "127.0.0.1:8080".parse()?;
//!     let mut buffer = vec![];
//!     let mut stream = TcpStream::connect(&socket_addr).await?;
//!
//!     stream.read(&mut buffer).await?;
//!     println!("{:?}", buffer);
//!     Ok(())
//! }
//! ```

mod tcp;
mod udp;
pub mod uds;

pub(in crate::net) mod reactor;

#[doc(inline)]
pub use crate::net::tcp::{TcpListener, TcpStream};
#[doc(inline)]
pub use crate::net::udp::UdpSocket;
#[doc(inline)]
pub use crate::net::uds::{UnixDatagram, UnixListener, UnixStream};
