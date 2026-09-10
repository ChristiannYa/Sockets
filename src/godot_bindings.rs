use crate::{
    FIELD_LENGTHS, PacketKind,
    bits::{BitReader, BitWriter, PacketSchema},
};
use godot::prelude::*;

struct BitpExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BitpExtension {}

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
struct GdPacketSchema {
    schema: PacketSchema<'static>,
}

#[godot_api]
impl GdPacketSchema {
    #[constant]
    const PACKET_KIND_SINGLE: u8 = PacketKind::Single as u8;
    #[constant]
    const PACKET_KIND_BATCH: u8 = PacketKind::Batch as u8;

    #[func]
    fn create() -> Gd<Self> {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        Gd::from_init_fn(|_base| Self { schema })
    }

    #[func]
    fn decode(&self, buf: PackedByteArray) -> VarDictionary {
        let mut reader = BitReader::new(&self.schema, buf.as_slice());
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
        let bytes: &[u8] = buf.as_slice();
        let mut buf_out = VarArray::new();

        let Some((&record_count, mut records)) = bytes.split_first() else {
            return buf_out; // Empty packet, nothing to decode
        };

        for _ in 0..record_count {
            let Some((&record_len, rem)) = records.split_first() else {
                // Truncated: packet is shorter than what its own header claims
                break;
            };
            let record_len = record_len as usize;
            if rem.len() < record_len {
                // Truncated: declared record_len overruns the buffer.
                // Without this check, next's `.split_at()` call (to split the
                // record's actual data and the remaining records) would
                // panic with index out of bounds
                break;
            }

            let (buf, rem) = rem.split_at(record_len);
            records = rem;

            // We could do `self.decode(buf.into())`, but that would build a fresh
            // `PackedByteArray` per record, causing a small heap allocation
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
    fn encode(&self, fields: VarDictionary) -> PackedByteArray {
        let mut writer = BitWriter::new(&self.schema);

        for (field_name, _) in self.schema.fields.iter() {
            if let Some(val) = fields.get(format!("{field_name:?}")) {
                let val: i64 = val.to();
                writer.write(field_name, &(val as u32));
            }
        }

        PackedByteArray::from(writer.buf().as_slice())
    }
}
