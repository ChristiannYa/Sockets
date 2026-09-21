pub mod bits;
pub mod fields;
pub mod quantize;
pub mod util;

#[cfg(feature = "godot")]
pub mod godot_bindings;

#[repr(u8)]
pub enum PacketType {
    Data,
    Ack,
}

#[repr(u8)]
pub enum DecodeType {
    Single,
    Batch,
}
