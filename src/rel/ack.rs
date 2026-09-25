#[repr(u8)]
pub enum PacketType {
    Data,
    Ack,
}

/// Returns `Some(id)` if `buf` has 2 bytes and the first one is the ack flag.
/// Returns `None` if the packet is malformed, so it won't panic the server.
pub fn decode(buf: &[u8]) -> Option<u8> {
    match buf {
        [kind, id] if *kind == PacketType::Ack as u8 => Some(*id),
        _ => None,
    }
}

/// Returns the packet being acknowledged with 2 bytes:
/// `PacketType::Ack` and `id`
pub fn encode(id: u8) -> [u8; 2] {
    [PacketType::Ack as u8, id]
}
