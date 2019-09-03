/// RCON Packet structure
#[derive(Default, Debug)]
pub struct Packet {
    pub request_id: i32,
    pub kind: u32,
    pub payload: Vec<u8>,
}

impl Packet {
    pub const EXEC_COMMAND: u32 = 2;
    pub const AUTHENTICATE: u32 = 3;
    pub const RESPONSEVALUE: u32 = 0;
    pub const AUTH_RESPONSE: u32 = 2;
    /// Create new packet with given request `Id`, `kind` and `payload`;
    /// ```
    /// let packet = Packet::new(1, Packet::AUTHENTICATE, b"let_me_in");
    /// ```
    ///
    /// TODO: check incomming payload and split to several packets if the packet length is to big.
    ///
    pub fn new(request_id: i32, kind: u32, string: &[u8]) -> Result<Self, String> {
        let payload: Vec<u8> = Vec::from(string);
        Ok(Packet {
            request_id,
            kind,
            payload,
        })
    }
    ///
    /// Generate auth packet
    /// ```
    /// let packet =  Packet::auth(1, b"let_me_in");
    /// ```
    pub fn auth(id: i32, password: &[u8]) -> Result<Self, String> {
        Self::new(id, Packet::AUTHENTICATE, password)
    }
    ///
    /// Generate empty packet
    /// ```
    /// let packet =  Packet::empty();
    /// ```
    pub fn empty() -> Self {
        Packet {
            request_id: 0,
            kind: 0,
            payload: Vec::new(),
        }
    }
    ///
    /// Convert current packet to bytes before send it by stream
    /// ```
    /// let packet =  Packet::auth(1, b"let_me_in");
    /// let bytes = packet.as_bytes();
    /// // now you can you it to write into stream
    /// ```
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(14 + self.payload.len());
        // Length
        for b in (10_u32 + self.payload.len() as u32).to_le_bytes().iter() {
            out.push(*b);
        }
        // Request ID
        for b in self.request_id.to_le_bytes().iter() {
            out.push(*b);
        }
        // Type
        for b in self.kind.to_le_bytes().iter() {
            out.push(*b);
        }
        // Payload
        for b in self.payload.clone() {
            out.push(b);
        }
        // Padding
        out.push(0x00);
        out.push(0x00);
        out
    }
}
use std::convert::TryInto;

pub fn read_le_i32(input: &mut &[u8]) -> i32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
    *input = rest;
    i32::from_le_bytes(int_bytes.try_into().unwrap())
}

pub fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
        .try_into()
        .unwrap()
}
