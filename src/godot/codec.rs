use crate::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    intent,
    pkt::{FIELD_LENGTHS, FieldName, PacketKind},
    sim::{PlayerIntentSim, PlayerIntentSimEdges, PlayerIntentSimHeld},
};
use godot::prelude::{
    Gd, GodotClass, PackedByteArray, VarDictionary, godot_api,
};

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpCodec {
    schema: PacketSchema,
}

#[godot_api]
impl UdpCodec {
    #[constant]
    const PKT_DATA: u8 = PacketKind::Data as u8;
    #[constant]
    const PKT_ACK: u8 = PacketKind::Ack as u8;
    #[constant]
    const PKT_INTENT: u8 = PacketKind::Intent as u8;
    #[constant]
    const PKT_SNAPSHOT: u8 = PacketKind::Snapshot as u8;

    #[constant]
    const DEC_SINGLE: u8 = DecodeType::Single as u8;
    #[constant]
    const DEC_BATCH: u8 = DecodeType::Batch as u8;

    #[func]
    fn create() -> Gd<Self> {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        Gd::from_init_fn(|_| Self { schema })
    }

    /// Only the `Dev*` fields of a single [`PacketKind::Data`] record that
    /// a player needs in order to start the game
    #[func]
    fn decode_dev(&self, buf: PackedByteArray) -> VarDictionary {
        // Strip [PacketType, DecodeType] — caller has already branched on
        // these to know this is a Data/Single packet before calling decode.
        let Some(body) = buf.as_slice().get(2..) else {
            return VarDictionary::new(); // too short to even hold the header
        };

        let mut reader = BitReader::new(&self.schema, body);
        let mut dict = VarDictionary::new();

        for (field_name, _) in self.schema.fields.iter() {
            if !reader.isset(field_name) {
                continue;
            }

            // Read every set field, even the ones skipped below, to keep the
            // reader's bit cursor in sync
            let val = reader.read(field_name);
            let is_dev = matches!(
                field_name,
                FieldName::DevSessionId | FieldName::DevPacketId
            );
            if is_dev {
                dict.set(format!("{field_name:?}"), val);
            }
        }
        dict
    }

    #[func]
    fn encode(&self, fields: VarDictionary) -> PackedByteArray {
        let mut writer = BitWriter::new(&self.schema);

        for (field_name, _) in self.schema.fields.iter() {
            if let Some(val) = fields.get(format!("{field_name:?}")) {
                let val: i64 = val.to();
                writer.write(field_name, &(val as u32));
            }
        }

        let mut buf = vec![PacketKind::Data as u8, DecodeType::Single as u8];
        buf.extend(writer.buf());
        PackedByteArray::from(buf.as_slice())
    }

    #[func]
    fn encode_intent(
        &self,
        seq: u8,
        mv_x: f32,
        mv_z: f32,
        yaw: f32,
        crouch: bool,
        jump_ct: u8,
    ) -> PackedByteArray {
        let held = PlayerIntentSimHeld { mv_x, mv_z, yaw, crouch };
        let edge_cts = PlayerIntentSimEdges { jump: jump_ct };
        let cmd = PlayerIntentSim::<u8> { held, edges: edge_cts };
        PackedByteArray::from(intent::encode(&self.schema, seq, &cmd))
    }
}
