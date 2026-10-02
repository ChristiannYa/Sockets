use std::cell::Cell;

use bitp::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    pkt::{
        PacketKind, {FIELD_LENGTHS, FieldName},
    },
};

pub enum Rel {
    Id(u8),
    Seq(u8),

    /// headless packs (e.g., sync records)
    None,
}

pub struct FieldsPack<'a> {
    pub id: Option<u8>,
    pub fields: &'a [(FieldName, u32)],
    pub sid: u32,
}

pub struct Codec {
    schema: PacketSchema,
    next_pkt_id: Cell<u8>,
}

impl Codec {
    pub fn new() -> Self {
        let schema: PacketSchema = PacketSchema::build(FIELD_LENGTHS).unwrap();

        Codec {
            schema,
            next_pkt_id: Cell::new(0),
        }
    }

    pub fn writer(&self) -> BitWriter<'_> {
        BitWriter::new(&self.schema)
    }

    pub fn reader<'b>(&self, buf: &'b [u8]) -> BitReader<'_, 'b> {
        BitReader::new(&self.schema, buf)
    }

    pub fn schema(&self) -> &PacketSchema {
        &self.schema
    }

    pub fn next_pkt_id(&self) -> u8 {
        let id = self.next_pkt_id.get();
        self.next_pkt_id.set(id.wrapping_add(1));
        id
    }

    /// Returns the 2-byte `[PacketType:: Data, decode_type]` header
    pub fn header_buf(&self, decode_type: DecodeType) -> Vec<u8> {
        vec![PacketKind::Data as u8, decode_type as u8]
    }

    /// For packing schema related values and with the header prepended.
    pub fn headful_pack(&self, args: FieldsPack) -> Vec<u8> {
        let mut buf = self.header_buf(DecodeType::Single);
        buf.extend(self.headless_pack(args));
        buf
    }

    /// For packing schema related values.
    ///
    /// *Contains `DevSessionId` by default*
    pub fn headless_pack(&self, args: FieldsPack) -> Vec<u8> {
        let mut writer = self.writer();
        self.seed_pkt(&mut writer, args.sid, args.id.map_or(Rel::None, Rel::Id));

        for (name, val) in args.fields {
            writer.write(name, val)
        }

        writer.buf()
    }

    pub fn pkt_ioids(&self, reader: &mut BitReader) -> (Option<u8>, Option<u8>) {
        let inid = reader
            .isset(&FieldName::DevPacketId)
            .then(|| reader.read(&FieldName::DevPacketId) as u8);
        let outid = inid.map(|_| self.next_pkt_id());
        (inid, outid)
    }

    pub fn seed_pkt(&self, writer: &mut BitWriter, sid: u32, rel: Rel) {
        writer.write(&FieldName::DevSessionId, &sid);
        match rel {
            Rel::Seq(seq) => writer.write(&FieldName::DevSequence, &(seq as u32)),
            Rel::Id(id) => writer.write(&FieldName::DevPacketId, &(id as u32)),
            Rel::None => {}
        }
    }
}
