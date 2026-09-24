use crate::rel::ack::PacketType;

/// Returns `Some(id)` if `buf` has 2 bytes and the first one is the ack flag.
/// Returns `None` if the packet is malformed, so it won't panic the server.
pub fn decode(buf: &[u8]) -> Option<u8> {
    match buf {
        [kind, id] if *kind == PacketType::Ack as u8 => Some(*id),
        _ => None,
    }
}
