use crate::{
    codec::{Codec, FieldsPack, Rel},
    world::{RelTarget, player::Players},
};
use bitp::{
    bits::{BitReader, BitWriter, DecodeType},
    logt,
    pkt::{FieldName, PacketKind},
    qua,
    rel::retx::PendingPacket,
};
use std::time::Instant;
use tap::Pipe;

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

                self.pk(FieldsPack {
                    id: None,
                    fields: &[
                        (FieldName::LocationX, loc_qua.x.encode(x)),
                        (FieldName::LocationY, qua::intent::Y.encode(y)),
                        (FieldName::LocationZ, loc_qua.z.encode(z)),
                        (FieldName::ColorH, hsv_qua.h.encode(p.color.h)),
                        (FieldName::ColorS, hsv_qua.s.encode(p.color.s)),
                        (FieldName::ColorV, hsv_qua.v.encode(p.color.v)),
                    ],
                    sid: *sid,
                })
            })
            .collect();

        if records.is_empty() {
            None
        } else {
            let mut buf =
                self.header(PacketKind::Data, Some(DecodeType::Batch), None);
            buf.extend(Self::records_buf(records));
            Some(buf)
        }
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
                self.pk(FieldsPack {
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
            None
        } else {
            let [lo, hi] = tick_ct.to_le_bytes();
            let mut buf = self.header(PacketKind::Snapshot, None, [lo, hi]);
            buf.extend(Self::records_buf(records));
            Some(buf)
        }
    }

    /// Reads every set schema field, writing each one into the writer's
    /// buffer.
    /// Returns a fresh packet that contains a header and the writer's updated
    /// buffer.
    pub fn set_in_schema_pkt(
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

        let mut buf =
            self.header(PacketKind::Data, Some(DecodeType::Single), None);
        buf.extend(writer.buf());
        buf
    }

    pub fn rel_pkting(
        &self,
        fields: &[(FieldName, u32)],
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let id = self.codec.next_pkt_id();

        let pkt = self
            .header(PacketKind::Data, Some(DecodeType::Single), None)
            .pipe(|mut buf| {
                buf.extend(self.pk(FieldsPack {
                    id: Some(id),
                    fields,
                    sid: rel_target.sid,
                }));
                buf
            });

        rel_target.pending_pkts.insert(
            (id, *rel_target.skt_src),
            PendingPacket::new(id, pkt.to_vec(), Instant::now()),
        );

        (pkt, id)
    }

    pub fn header<I>(
        &self,
        kind: PacketKind,
        dec: Option<DecodeType>,
        extra: I,
    ) -> Vec<u8>
    where
        I: IntoIterator<Item = u8>,
    {
        let extra = extra.into_iter();
        let mut buf = Vec::with_capacity(2 + extra.size_hint().0);

        buf.push(kind as u8);
        if let Some(dec) = dec {
            buf.push(dec as u8)
        }
        buf.extend(extra);

        buf
    }

    /// Pack schema-only values on a seeded buffer (seeded internally).
    fn pk(&self, args: FieldsPack) -> Vec<u8> {
        let mut writer = self.codec.writer();

        self.codec.seed_pkt(
            &mut writer,
            args.sid,
            args.id.map_or(Rel::None, Rel::Id),
        );

        for (name, val) in args.fields {
            writer.write(name, val)
        }

        writer.buf()
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
