pub mod logic;
pub mod player;
mod state;

use crate::{
    codec::{Codec, FieldsPack},
    rel::retx::PendingPackets,
    world::{player::Players, state::WorldState},
};
use bitp::{
    bits::{BitReader, BitWriter, DecodeType},
    logt,
    pkt::FieldName,
    quantize,
    rel::retx::PendingPacket,
};
use std::{net::SocketAddr, time::Instant};

/// Who to send to and where to track it
pub struct RelTarget<'r> {
    pub sid: u32,
    pub skt_src: &'r SocketAddr,
    pub pending_pkts: &'r mut PendingPackets,
}

pub struct World<'w> {
    /// Legacy relay, will be deprecated in the future
    state: WorldState,

    codec: &'w Codec,
    pub players: Players,
}

impl<'w> World<'w> {
    pub fn new(codec: &'w Codec) -> Self {
        World {
            state: WorldState::new(),
            codec,
            players: Players::default(),
        }
    }

    /// Returns a reliable buffer with the [FieldName::DevIsNewPlayer] flag
    /// Information: [DecodeType::Single], [FieldName::DevSessionId] and
    /// [FieldName::DevIsNewPlayer]
    pub fn welcome_buf(&self, rel_target: &mut RelTarget) -> Vec<u8> {
        let (buf, _) =
            self.rel_track(&[(FieldName::DevIsNewPlayer, 1)], rel_target);
        buf
    }

    /// Spawns player in the simulation
    /// Returns a reliable buffer along with its id containing essential spawn
    /// information (location and color)
    pub fn spawn_player(
        &mut self,
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let (spawn_pt, spawn_pt_codec) = (
            logic::spawn::player_spawn_pt(),
            quantize::loc(self.codec.schema()),
        );
        let (hsv, hsv_codec) = (
            logic::color::hsv(),
            quantize::hsv(self.codec.schema()), //
        );

        let (enc_x, enc_z) = (
            spawn_pt_codec.x.encode(spawn_pt.x),
            spawn_pt_codec.z.encode(spawn_pt.z),
        );

        let fields = [
            (FieldName::LocationX, enc_x),
            (FieldName::LocationZ, enc_z),
            (FieldName::ColorH, hsv_codec.h.encode(hsv.h)),
            (FieldName::ColorS, hsv_codec.s.encode(hsv.s)),
            (FieldName::ColorV, hsv_codec.v.encode(hsv.v)),
        ];

        for (field_name, val) in fields.iter() {
            self.state
                .save(field_name, rel_target.sid, *val)
        }

        self.players.spawn(
            rel_target.sid,
            spawn_pt_codec.x.decode(enc_x),
            spawn_pt_codec.z.decode(enc_z),
        );

        self.rel_track(&fields, rel_target)
    }

    fn rel_track(
        &self,
        fields: &[(FieldName, u32)],
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let id = self.codec.next_pkt_id();
        let buf = self.codec.headful_pack(FieldsPack {
            id: Some(id),
            fields,
            sid: rel_target.sid,
        });

        rel_target.pending_pkts.insert(
            (id, *rel_target.skt_src),
            PendingPacket::new(id, buf.to_vec(), Instant::now()),
        );

        (buf, id)
    }

    /// Reads every field out of `buf`, echoing each one into a new packet for
    /// the broadcast buffer
    pub fn decode(
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
            self.state.save(field_name, sid, val);

            logt!("#{sid}: {field_name:?}={val}")
        }

        let mut buf = self.codec.header_buf(DecodeType::Single);
        buf.extend(writer.buf());
        buf
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
    pub fn sync_buf(&self, sid: u32) -> Vec<u8> {
        let records: Vec<Vec<u8>> = self
            .state
            .world
            .iter()
            .filter(|(sid_iter, _)| **sid_iter != sid)
            .map(|(sid, player_state)| {
                let mut fields = Vec::<(FieldName, u32)>::new();

                for (field_name, _) in self.codec.schema().fields.iter() {
                    if let Some(val) = player_state.get(field_name) {
                        fields.push((field_name.clone(), *val));
                    }
                }

                self.codec.headless_pack(FieldsPack {
                    id: None,
                    fields: &fields,
                    sid: *sid,
                })
            })
            .collect();

        self.batch_pack(records)
    }

    pub fn players_snapshot_buf(&self) -> Option<Vec<u8>> {
        let loc_qua = quantize::loc(self.codec.schema());

        let records: Vec<Vec<u8>> = self
            .players
            .iter()
            .map(|(sid, player)| {
                let [x, _, z] = player.state.pos;
                self.codec.headless_pack(FieldsPack {
                    id: None,
                    fields: &[
                        (FieldName::LocationX, loc_qua.x.encode(x)),
                        (FieldName::LocationZ, loc_qua.z.encode(z)),
                    ],
                    sid: *sid,
                })
            })
            .collect();

        if records.is_empty() {
            return None;
        }

        Some(self.batch_pack(records))
    }

    fn batch_pack(&self, records: Vec<Vec<u8>>) -> Vec<u8> {
        let mut buf = self.codec.header_buf(DecodeType::Batch);
        buf.push(records.len() as u8);
        for record in records {
            buf.push(record.len() as u8);
            buf.extend(record);
        }
        buf
    }

    pub fn has_state(&self) -> bool { !self.state.world.is_empty() }
}
