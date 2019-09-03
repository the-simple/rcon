use crate::protocol::Packet;
use std::io::prelude::*;

pub mod tcp;

#[derive(Debug, Default)]
pub struct RconClient<T>
where
    T: Read + Write,
{
    id: i32,
    pub stream: T,
}
impl<T> RconClient<T>
where
    T: Read + Write,
{
    pub fn new(stream: T) -> Self {
        RconClient { id: 0, stream }
    }
    pub fn next_id(&mut self) -> i32 {
        self.id += 1;
        self.id
    }
    pub fn write_packet(&mut self, packet: Packet) -> std::io::Result<usize> {
        debug!("Write packet {:?}", packet);
        trace!("Packet as bytes: {:?}", packet.as_bytes());
        let ret = self.stream.write(&packet.as_bytes())?;
        self.stream.flush()?;
        debug!("{:?} bytes were written.", ret);
        Ok(ret)
    }
    pub fn read_packet(&mut self) -> std::io::Result<Packet> {
        let packet = {
            let response_length = {
                let mut buf = [0; 4];
                self.stream.read_exact(&mut buf)?;
                trace!("Length buffer: {:?}", buf);
                let out = u32::from_le_bytes(buf);
                debug!("Packet length =  {:?}", out);
                out
            };
            let request_id = {
                let mut buf = [0; 4];
                self.stream.read_exact(&mut buf)?;
                trace!("ID buffer: {:?}", buf);
                let out = i32::from_le_bytes(buf);
                debug!("Packet ID =  {:?}", out);
                out
            };
            let request_type = {
                let mut buf = [0; 4];
                self.stream.read_exact(&mut buf)?;
                trace!("Request type buffer: {:?}", buf);
                let out = u32::from_le_bytes(buf);
                debug!("Packet Type =  {:?}", out);
                out
            };
            let payload = {
                let payload_capacity = response_length as usize - 10usize;
                trace!("PAYLOAD capacity is {:?}", payload_capacity);
                if payload_capacity < 1 {
                    String::new()
                } else {
                    let mut buf: Vec<u8> = Vec::with_capacity(payload_capacity);
                    // expand a vector to be able fill it with incomming data
                    // without it `read_exact` reads 0 bytes because the length of the Vec is 0
                    buf.resize_with(payload_capacity, || 0x00);
                    self.stream.read_exact(&mut buf)?;
                    trace!(
                        "Payload buffer: {:?} with lenght = {}  and capacity = {}",
                        buf,
                        buf.len(),
                        buf.capacity()
                    );
                    let out = String::from_utf8(buf).unwrap();
                    debug!("Output PAYLOAD: {:?}", out);
                    out
                }
            };
            {
                let mut buf = [0; 2];
                self.stream.read_exact(&mut buf)?;
                trace!("Padding Buffer: {:?}", buf);
            };
            Packet::new(request_id, request_type, payload.as_bytes())
                .expect("I cannot create packet")
        };
        debug!("Read packet: {:?}", packet);
        Ok(packet)
    }
    pub fn auth(&mut self, password: &[u8]) -> std::io::Result<Packet> {
        let packet = Packet::auth(self.next_id(), password).expect("I cannot create packet");
        trace!("Auth packet request {:?}", packet);
        self.write_packet(packet)?;
        let packet = self.read_packet();
        packet
    }

    pub fn command(&mut self, command: &[u8]) -> std::io::Result<Packet> {
        trace!("Send a command {:?}", command);
        let command_id = self.next_id();
        let packet =
            Packet::new(command_id, Packet::EXEC_COMMAND, command).expect("I cannot create packet");
        self.write_packet(packet)?;
        let packet = self.read_packet();
        packet
    }
}
