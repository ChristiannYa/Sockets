use crate::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    fields::FIELD_LENGTHS,
    quantize::{hsv::hsv_codec, loc::loc_codec},
    rel::ack::PacketType,
};
use godot::{
    meta::ToGodot,
    prelude::{
        Color, Gd, GodotClass, PackedByteArray, VarArray, VarDictionary, Vector3, godot_api,
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
    const PKT_DATA: u8 = PacketType::Data as u8;
    #[constant]
    const PKT_ACK: u8 = PacketType::Ack as u8;

    #[constant]
    const DEC_SINGLE: u8 = DecodeType::Single as u8;
    #[constant]
    const DEC_BATCH: u8 = DecodeType::Batch as u8;

    #[func]
    fn create() -> Gd<Self> {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        Gd::from_init_fn(|_| Self { schema })
    }

    #[func]
    fn decode(&self, buf: PackedByteArray) -> VarDictionary {
        // Strip [PacketType, DecodeType] — caller has already branched on
        // these to know this is a Data/Single packet before calling decode.
        let Some(body) = buf.as_slice().get(2..) else {
            return VarDictionary::new(); // too short to even hold the header
        };

        let mut reader = BitReader::new(&self.schema, body);
        let mut dict = VarDictionary::new();

        for (field_name, _) in self.schema.fields.iter() {
            if reader.isset(field_name) {
                dict.set(format!("{field_name:?}"), reader.read(field_name));
            }
        }

        dict
    }

    #[func]
    fn decode_batch(&self, buf: PackedByteArray) -> VarArray {
        let mut buf_out = VarArray::new();

        // Strip [PacketType, DecodeType] before the record-count byte
        let Some(bytes) = buf.as_slice().get(2..) else {
            return buf_out;
        };

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

            let mut reader = BitReader::new(&self.schema, buf);
            let mut dict = VarDictionary::new();
            for (field_name, _) in self.schema.fields.iter() {
                if reader.isset(field_name) {
                    dict.set(format!("{field_name:?}"), reader.read(field_name));
                }
            }
            buf_out.push(&dict.to_variant());
        }

        buf_out
    }

    #[func]
    fn decode_loc(&self, x: u32, z: u32) -> Vector3 {
        let codec = loc_codec(&self.schema);
        Vector3::new(codec.x.decode(x), 0.0, codec.z.decode(z))
    }

    #[func]
    fn decode_hsv(&self, h: u32, s: u32, v: u32) -> Color {
        let codec = hsv_codec(&self.schema);
        Color::from_hsv(
            codec.h.decode(h) as f64,
            codec.s.decode(s) as f64,
            codec.v.decode(v) as f64,
        )
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

        let mut buf = vec![PacketType::Data as u8, DecodeType::Single as u8];
        buf.extend(writer.buf());
        PackedByteArray::from(buf.as_slice())
    }
}
