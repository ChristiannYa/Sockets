mod prog;
pub mod reader;
pub mod writer;

#[repr(u8)]
pub enum DecodeType {
    Single,
    Batch,
}

#[repr(u8)]
pub enum PacketType {
    Data,
    Ack,
}
