use godot::prelude::{GodotClass, PackedByteArray, godot_api};

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpAck {}

#[godot_api]
impl UdpAck {
    #[func]
    fn decode_ack(buf: PackedByteArray) -> i32 {
        crate::rel::ack::decode(buf.as_slice())
            .map(|id| id as i32)
            .unwrap_or(-1)
    }

    #[func]
    fn encode_ack(id: u8) -> PackedByteArray {
        PackedByteArray::from(crate::rel::ack::encode(id).as_slice())
    }
}
