use crate::rel::ack::PacketType;

/// Returns the packet being acknowledged with 2 bytes:
/// `PacketType::Ack` and `id`
pub fn encode(id: u8) -> [u8; 2] {
    [PacketType::Ack as u8, id]
}
