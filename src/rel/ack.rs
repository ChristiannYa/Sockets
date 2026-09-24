mod codec;

pub use codec::decode::decode;
pub use codec::encode::encode;

#[repr(u8)]
pub enum PacketType {
    Data,
    Ack,
}
