#[macro_use]
extern crate log;

pub mod client;
pub mod protocol;

pub mod prelude {
    pub use super::protocol::Packet;
    pub use super::client::tcp::RconClient;
}

#[cfg(test)]
mod tests;
