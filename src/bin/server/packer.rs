use bitp::{
    FIELD_LENGTHS, PacketKind,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};

use crate::world::state::ClientStates;

pub struct Packer<'a> {
    schema: PacketSchema<'a>,
}

impl<'a> Packer<'a> {
    /// Panics if construction of [PacketSchema] fails
    pub fn build() -> Self {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        Packer { schema }
    }

    /// Reads every field out of `buf`, echoing each one into a new packet seeded
    /// with `sid` and `seq` for the broadcast.
    /// Returns (decoded fields, broadcast buffer).
    pub fn process(&self, buf: &[u8], sid: u32, seq: u8) -> (Vec<(FieldName, u32)>, Vec<u8>) {
        let mut reader = BitReader::new(&self.schema, buf);
        let mut writer = BitWriter::new(&self.schema);

        self.seed(&mut writer, sid, seq);
        let decoded: Vec<(FieldName, u32)> = self.decode(&mut reader, &mut writer);

        let mut buf = vec![PacketKind::Single as u8];
        buf.extend(writer.buf());

        (decoded, buf)
    }

    fn decode(
        &self,
        reader: &mut BitReader<'a>,
        writer: &mut BitWriter<'a>,
    ) -> Vec<(FieldName, u32)> {
        let mut packed = Vec::<(FieldName, u32)>::new();

        for (field_name, _) in self.schema.fields.iter() {
            if !reader.isset(field_name) {
                continue;
            }

            let value = reader.read(field_name);
            writer.write(field_name, &value);
            packed.push((field_name.clone(), value));
        }

        packed
    }

    /// Seeds writer's buffer with [FieldName]'s `SessionId` and `Sequence`
    /// regardless of client's newness
    fn seed(&self, writer: &mut BitWriter<'a>, sid: u32, seq: u8) {
        writer.write(&FieldName::SessionId, &sid);
        writer.write(&FieldName::Sequence, &(seq as u32));
    }

    /// Returns a packet with information indicating that the client's is new.
    /// Information: [PacketKind]'s `Single`, [FieldName]'s `SessionId` and `IsNewClient`
    pub fn welcome(&self, new_cli_sid: u32) -> Vec<u8> {
        let mut buf = vec![PacketKind::Single as u8];

        let mut writer = BitWriter::new(&self.schema);
        writer.write(&FieldName::SessionId, &new_cli_sid);
        writer.write(&FieldName::IsNewClient, &1);

        buf.extend(writer.buf());
        buf
    }

    /// Returns a buffer containing every other connected client's
    /// state to sync a newly-joined client.
    /// Layout:
    /// ```
    /// let record_count = vec![2];
    ///
    /// // 1=record_len, 2=mask_bytes, 3=value_bytes
    /// // `record_len` is the byte length of the client's `BitWriter::buf()`'s
    /// // output, including `SessionId` as a "seed" value to identify the record
    /// let record1 = vec![1, 2, 3];
    /// let record2 = vec![1, 2, 3];
    ///
    /// // [2, 1, 2, 3, 1, 2, 3]
    /// return [record_count, record1, record2].concat()
    /// ```
    pub fn sync(&self, cli_states: &ClientStates, new_cli_sid: u32) -> Vec<u8> {
        let records: Vec<Vec<u8>> = cli_states
            .iter()
            .filter(|(sid, _)| **sid != new_cli_sid)
            .map(|(sid, cli_state)| {
                let mut writer = BitWriter::new(&self.schema);
                writer.write(&FieldName::SessionId, sid);

                for (field_name, _) in self.schema.fields.iter() {
                    if let Some(val) = cli_state.get(field_name) {
                        writer.write(field_name, val);
                    }
                }

                writer.buf()
            })
            .collect();

        let mut buf_sync = vec![PacketKind::Batch as u8, records.len() as u8];
        for record in records {
            buf_sync.push(record.len() as u8);
            buf_sync.extend(record);
        }
        buf_sync
    }

    pub fn schema(&self) -> &PacketSchema<'a> {
        &self.schema
    }
}
