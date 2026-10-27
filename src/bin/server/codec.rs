use std::cell::Cell;

use bitp::{
    bits::{BitReader, BitWriter, PacketSchema},
    pkt::{FIELD_LENGTHS, FieldName},
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

        Codec { schema, next_pkt_id: Cell::new(0) }
    }

    pub fn writer(&self) -> BitWriter<'_> { BitWriter::new(&self.schema) }

    pub fn reader<'b>(&self, buf: &'b [u8]) -> BitReader<'_, 'b> {
        BitReader::new(&self.schema, buf)
    }

    pub fn schema(&self) -> &PacketSchema { &self.schema }

    pub fn next_pkt_id(&self) -> u8 {
        let id = self.next_pkt_id.get();
        self.next_pkt_id.set(id.wrapping_add(1));
        id
    }

    pub fn pkt_ioids(
        &self,
        reader: &mut BitReader,
    ) -> (Option<u8>, Option<u8>) {
        let inid = reader
            .isset(&FieldName::DevPktId)
            .then(|| reader.read(&FieldName::DevPktId) as u8);
        let outid = inid.map(|_| self.next_pkt_id());
        (inid, outid)
    }

    /// Writes [FieldName::DevSid] by default and a written reliable
    /// value of either [Rel::Seq] or [Rel::Id], or no value written at all
    /// if [Rel::None] is set
    pub fn seed_pkt(&self, writer: &mut BitWriter, sid: u32, rel: Rel) {
        writer.write(&FieldName::DevSid, &sid);
        match rel {
            Rel::Seq(seq) => writer.write(&FieldName::DevSeq, &(seq as u32)),
            Rel::Id(id) => writer.write(&FieldName::DevPktId, &(id as u32)),
            Rel::None => {}
        }
    }
}
