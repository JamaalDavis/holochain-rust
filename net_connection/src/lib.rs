#[macro_use]
extern crate failure;
extern crate rmp_serde;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;

use failure::Error;

pub type NetResult<T> = Result<T, Error>;

pub mod protocol;
pub mod net_connection;
pub mod net_connection_thread;