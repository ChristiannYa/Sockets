use crate::{
    codec::{Codec, FieldsPack},
    world::{RelTarget, player::Players},
};
use bitp::{
    bits::{BitReader, BitWriter, DecodeType},
    logt,
    pkt::FieldName,
    qua,
    rel::retx::PendingPacket,
};
use std::time::Instant;

pub struct WorldPacker<'w> {
    codec: &'w Codec,
}

impl<'w> WorldPacker<'w> {
    pub fn new(codec: &'w Codec) -> Self { WorldPacker { codec } }

    /// Returns a reliable packet with the [FieldName::DevIsNewPlayer] flag
    /// Information: [DecodeType::Single], [FieldName::DevSessionId] and
    /// [FieldName::DevIsNewPlayer]
    pub fn welcome_pkt(&self, rel_target: &mut RelTarget) -> Vec<u8> {
        let (pkt, _) =
            self.rel_pkting(&[(FieldName::DevIsNewPlayer, 1)], rel_target);
        pkt
    }

    /// Returns a buffer containing every other connected player's state to sync a
    /// newly-joined player.
    ///
    /// Layout:
    /// ```
    /// let record_count = vec![2];
    ///
    /// // 1=record_len, 2=mask_bytes, 3=value_bytes
    /// // `record_len` is the byte length of the player's `BitWriter::buf()`'s
    /// // output, including `SessionId` as a "seed" value to identify the record
    /// let record1 = vec![1, 2, 3];
    /// let record2 = vec![1, 2, 3];
    ///
    /// // [2, 1, 2, 3, 1, 2, 3]
    /// return [record_count, record1, record2].concat()
    /// ```
    pub fn sync_pkt(&self, sid: u32, players: &Players) -> Option<Vec<u8>> {
        let loc_qua = qua::loc(self.codec.schema());
        let hsv_qua = qua::hsv(self.codec.schema());

        let records: Vec<Vec<u8>> = players
            .iter()
            .filter(|(sid_iter, _)| **sid_iter != sid)
            .map(|(sid, p)| {
                let [x, y, z] = p.state.pos;

                self.codec.headless_pack(FieldsPack {
                    id: None,
                    fields: &[
                        (FieldName::LocationX, loc_qua.x.encode(x)),
                        (FieldName::LocationY, loc_qua.x.encode(y)),
                        (FieldName::LocationZ, loc_qua.z.encode(z)),
                        (FieldName::ColorH, hsv_qua.h.encode(p.color.h)),
                        (FieldName::ColorS, hsv_qua.s.encode(p.color.s)),
                        (FieldName::ColorV, hsv_qua.v.encode(p.color.v)),
                    ],
                    sid: *sid,
                })
            })
            .collect();

        if records.is_empty() { None } else { Some(self.batch_pack(records)) }
    }

    pub fn players_snapshot_pkt(
        &self,
        tick_ct: u16,
        players: &Players,
    ) -> Option<Vec<u8>> {
        let loc_qua = qua::loc(self.codec.schema());

        let records: Vec<Vec<u8>> = players
            .iter()
            .map(|(sid, player)| {
                let [x, y, z] = player.state.pos;
                self.codec.headless_pack(FieldsPack {
                    id: None,
                    fields: &[
                        (FieldName::LocationX, loc_qua.x.encode(x)),
                        (FieldName::LocationZ, loc_qua.z.encode(z)),
                        (FieldName::LocationY, qua::intent::Y.encode(y)),
                        (
                            FieldName::Yaw,
                            qua::intent::YAW.encode(player.state.yaw),
                        ),
                    ],
                    sid: *sid,
                })
            })
            .collect();

        if records.is_empty() {
            return None;
        }

        Some(self.snapshot_pack(records, tick_ct))
    }

    /// Reads every set schema field, echoing each one into a new packet for
    /// the broadcast buffer
    pub fn schema_pkt(
        &mut self,
        sid: u32,
        reader: &mut BitReader,
        writer: &mut BitWriter,
    ) -> Vec<u8> {
        for (field_name, _) in self.codec.schema().fields.iter() {
            if !reader.isset(field_name)
                || *field_name == FieldName::DevPacketId
            {
                continue;
            };

            let val = reader.read(field_name);
            writer.write(field_name, &val);

            logt!("#{sid}: {field_name:?}={val}")
        }

        let mut buf = self.codec.header_buf(DecodeType::Single);
        buf.extend(writer.buf());
        buf
    }

    pub fn rel_pkting(
        &self,
        fields: &[(FieldName, u32)],
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let id = self.codec.next_pkt_id();
        let pkt = self.codec.headful_pack(FieldsPack {
            id: Some(id),
            fields,
            sid: rel_target.sid,
        });

        rel_target.pending_pkts.insert(
            (id, *rel_target.skt_src),
            PendingPacket::new(id, pkt.to_vec(), Instant::now()),
        );

        (pkt, id)
    }

    /// `[Data, Batch]` + [World::records_buf]
    fn batch_pack(&self, records: Vec<Vec<u8>>) -> Vec<u8> {
        let mut buf = self.codec.header_buf(DecodeType::Batch);
        buf.extend(Self::records_buf(records));
        buf
    }

    /// `[Snapshot, tick_lo, tick_hi]` + [World::records_buf]
    fn snapshot_pack(&self, records: Vec<Vec<u8>>, tick_ct: u16) -> Vec<u8> {
        let mut buf = self.codec.snapshot_header_buf(tick_ct);
        buf.extend(Self::records_buf(records));
        buf
    }

    /// `[record_ct, (record_len, record)...]`, shared by every packet that
    /// carries records
    fn records_buf(records: Vec<Vec<u8>>) -> Vec<u8> {
        let mut buf = vec![records.len() as u8];
        for record in records {
            buf.push(record.len() as u8);
            buf.extend(record);
        }
        buf
    }
}
