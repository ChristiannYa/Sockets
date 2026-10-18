use crate::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    intent,
    pkt::{FIELD_LENGTHS, FieldName, PacketKind},
    sim::{PlayerIntentSim, PlayerIntentSimEdges, PlayerIntentSimHeld},
};
use godot::{
    meta::ToGodot,
    prelude::{
        Gd, GodotClass, PackedByteArray, VarArray, VarDictionary, godot_api,
    },
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

    /// Only the `Dev*` fields of a single Data packet's record
    #[func]
    fn decode_dev(&self, buf: PackedByteArray) -> VarDictionary {
        // Strip [PacketType, DecodeType] — caller has already branched on
        // these to know this is a Data/Single packet before calling decode.
        let Some(body) = buf.as_slice().get(2..) else {
            return VarDictionary::new(); // too short to even hold the header
        };

        self.dev_fields(body)
    }

    /// Only the `Dev*` fields of a single Data packet's record
    #[func]
    fn decode_dev_batch(&self, buf: PackedByteArray) -> VarArray {
        // Strip [PacketType, DecodeType] before the record-count byte
        let Some(bytes) = buf.as_slice().get(2..) else {
            return VarArray::new();
        };
        self.decode_dev_records(bytes)
    }

    /// Returns `{ "tick": int, "records": Array }`, with each record holding
    /// only its `Dev*` fields. Empty when the packet is too short to hold
    /// the heade
    #[func]
    fn decode_dev_snapshot(&self, buf: PackedByteArray) -> VarDictionary {
        let mut out = VarDictionary::new();

        // [PacketKind::Snapshot, tick_lo, tick_hi] + records buf
        let bytes = buf.as_slice();
        let (Some(&lo), Some(&hi), Some(records)) =
            (bytes.get(1), bytes.get(2), bytes.get(3..))
        else {
            return out;
        };

        out.set("tick".to_string(), u16::from_le_bytes([lo, hi]) as u32);
        out.set(
            "records".to_string(),
            &self.decode_dev_records(records).to_variant(),
        );
        out
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

impl UdpCodec {
    // @TODO
    // Only the `Dev*` fields are kept, for the script-side handler (sequence,
    // acks, new-player flag). Remove this and `decode`/`decode_snapshot`
    // together with that logic once it moves to Rust.
    fn decode_dev_records(&self, bytes: &[u8]) -> VarArray {
        let mut buf_out = VarArray::new();

        let Some((&record_count, mut records)) = bytes.split_first() else {
            return buf_out; // Empty packet, nothing to decode
        };

        for _ in 0..record_count {
            let Some((&record_len, rem)) = records.split_first() else {
                break;
            };
            let record_len = record_len as usize;
            if rem.len() < record_len {
                break;
            }

            let (buf, rem) = rem.split_at(record_len);
            records = rem;

            buf_out.push(&self.dev_fields(buf).to_variant());
        }

        buf_out
    }

    fn dev_fields(&self, record: &[u8]) -> VarDictionary {
        let mut reader = BitReader::new(&self.schema, record);
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
                FieldName::DevSessionId
                    | FieldName::DevSequence
                    | FieldName::DevPing
                    | FieldName::DevPacketId
                    | FieldName::DevIsNewPlayer
            );
            if is_dev {
                dict.set(format!("{field_name:?}"), val);
            }
        }
        dict
    }
}
