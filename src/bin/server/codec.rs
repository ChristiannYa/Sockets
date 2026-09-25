use std::cell::Cell;

use bitp::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    fields::{FIELD_LENGTHS, FieldName},
};
use tap::Pipe;

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

    /// Pack arbitrary field values into a fresh buffer
    pub fn pack(&self, fields: &[(FieldName, u32)], sid: u32) -> Vec<u8> {
        let mut writer = self.writer();
        writer.write(&FieldName::DevSessionId, &sid);

        for (name, val) in fields {
            writer.write(name, val);
        }

        let mut buf = vec![DecodeType::Single as u8];
        buf.extend(writer.buf());
        buf
    }

    /// Pre processes a packet by doing the following:
    ///
    /// - Checking if the packet is reliable
    /// - Seeding packet with the session id (unconditionally) and the sequence
    ///   (unless the packet is reliable)
    ///
    /// Returns the optional packet ids (in and out), reader, and writer
    /// which already read the packet in order to avoid any further
    /// redundant packet re-reads.
    pub fn preprocess_pkt<'b>(
        &self,
        buf: &'b [u8],
        sid: u32,
        seq: u8,
    ) -> (Option<(u8, u8)>, BitReader<'_, 'b>, BitWriter<'_>) {
        let (mut reader, mut writer) = (self.reader(buf), self.writer());

        let pkt_io_ids = reader
            .isset(&FieldName::DevPacketId)
            .then(|| reader.read(&FieldName::DevPacketId) as u8)
            .map(|id| (id, self.next_pkt_id()));

        self.seed_pkt(&mut writer, sid, seq, pkt_io_ids);

        (pkt_io_ids, reader, writer)
    }

    fn seed_pkt(&self, writer: &mut BitWriter, sid: u32, seq: u8, pkt_io_ids: Option<(u8, u8)>) {
        writer.write(&FieldName::DevSessionId, &sid);

        if pkt_io_ids.is_none() {
            writer.write(&FieldName::DevSequence, &(seq as u32))
        }

        if let Some((_, id_out)) = pkt_io_ids {
            writer.write(&FieldName::DevPacketId, &(id_out as u32));
        }
    }
}
