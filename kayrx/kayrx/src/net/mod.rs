//! # Async network TCP, UDP, UDS
//!
//! The types defined in this module are designed to closely follow the APIs of the
//! analogous types in `std::net`. But rather than implementing synchronous traits
//! like `std::io::{Read, Write}`, these types implement the asychronous versions
//! provided by the `futures-preview` crate, i.e. `futures::io::{AsyncRead, AsyncWrite}`.
//! When using `async`/`await` syntax, the experience should be quite similar to
//! traditional blocking code that uses `std::net`.
//!
//! Because futures-preview is currently unstable, this crate requires
//! nightly Rust.
//!
//! # Examples
//! __TCP Server__
//! ```rust
//! use kayrx::net::tcp::{TcpListener, TcpStream};
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
//! use kayrx::net::tcp::{TcpListener, TcpStream};
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

pub(in crate::net)  mod reactor;

#[doc(inline)]
pub use crate::net::tcp::{TcpListener, TcpStream};
#[doc(inline)]
pub use crate::net::udp::UdpSocket;
#[doc(inline)]
pub use crate::net::uds::{UnixDatagram, UnixListener, UnixStream};
