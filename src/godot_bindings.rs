use godot::prelude::*;
use crate::{FIELD_LENGTHS, bits::{BitReader, BitWriter, PacketSchema}};

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
                dict.set(
                    format!("{field_name:?}"), 
                    reader.read(field_name)
                );
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

        PackedByteArray::from(writer.buf().as_slice())
    }
}
