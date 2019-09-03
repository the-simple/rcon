use crate::protocol::Packet;
use crate::client::RconClient as Base;
use std::net::{Shutdown, TcpStream};

/// TcpSTream implementarion of base RCON protocol
#[derive(Debug)]
pub struct RconClient {
    rcon: Base<TcpStream>,
}

impl RconClient {
    pub fn new(host: String, port: Option<u16>) -> Result<RconClient, std::io::Error> {
        let port = port.unwrap_or(25575);
        let stream = TcpStream::connect((host.as_str(), port));
        match stream {
            Ok(s) => {
                let rcon = Base::new(s);
                Ok(RconClient { rcon })
            }
            Err(e) => {
                error!("Connection to {}:{} is not available", host, port);
                Err(e)
            }
        }
    }

    pub fn auth(&mut self, password: Option<String>) -> bool {
        let password = password.unwrap();
        let packet = self.rcon.auth(password.as_bytes()).unwrap();
        packet.kind == Packet::AUTH_RESPONSE
    }

    pub fn run(&mut self, command: String) -> Result<Packet, std::io::Error> {
        self.rcon.command(command.as_bytes())
    }

    pub fn reconnect(&mut self) -> Result<(), std::io::Error> {
        match self.rcon.stream.try_clone() {
            Ok(stream) => self.rcon.stream = stream,
            Err(_) => (),
        }
        Ok(())
    }
}

impl Drop for RconClient {
    fn drop(&mut self) {
        self.rcon
            .stream
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
    }
}
