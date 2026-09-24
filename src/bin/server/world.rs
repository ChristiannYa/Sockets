pub mod logic;

use crate::codec::Codec;
use bitp::{
    bits::{BitReader, BitWriter, DecodeType},
    fields::FieldName,
    quantize::{hsv::hsv_codec, loc::loc_codec},
};
use std::collections::HashMap;

type WorldState = HashMap<u32, HashMap<FieldName, u32>>;

pub struct World<'c> {
    state: State,
    codec: &'c Codec,
}

impl<'c> World<'c> {
    pub fn new(codec: &'c Codec) -> Self {
        World {
            state: State::new(),
            codec,
        }
    }

    /// Returns a buffer with information indicating that the player is new.
    /// Information: [DecodeType]'s `Single`, [FieldName]'s `SessionId` and `IsNewPlayer`
    pub fn welcome_buf(&self, sid: u32) -> Vec<u8> {
        self.codec.pack(&[(FieldName::DevIsNewPlayer, 1)], sid)
    }

    /// Returns a buffer with spawn information
    /// - X and Z location of the player
    /// - HSV color of the player
    pub fn player_spawn_buf(&mut self, sid: u32) -> Vec<u8> {
        let (spawn_pt, spawn_pt_codec) = (
            logic::spawn::player_spawn_pt(),
            loc_codec(self.codec.schema()),
        );
        let (hsv, hsv_codec) = (
            logic::color::hsv(), //
            hsv_codec(self.codec.schema()),
        );

        let fields = [
            (FieldName::LocationX, spawn_pt_codec.x.encode(spawn_pt.x)),
            (FieldName::LocationZ, spawn_pt_codec.z.encode(spawn_pt.z)),
            (FieldName::ColorH, hsv_codec.h.encode(hsv.h)),
            (FieldName::ColorS, hsv_codec.s.encode(hsv.s)),
            (FieldName::ColorV, hsv_codec.v.encode(hsv.v)),
        ];

        for (field_name, val) in fields.iter() {
            self.state.save(field_name, sid, *val)
        }

        self.codec.pack(&fields, sid)
    }

    /// Reads every field out of `buf`, echoing each one into a new packet for
    /// the broadcast buffer
    pub fn process(&mut self, sid: u32, reader: &mut BitReader, writer: &mut BitWriter) -> Vec<u8> {
        for (field_name, _) in self.codec.schema().fields.iter() {
            if !reader.isset(field_name) || *field_name == FieldName::DevPacketId {
                continue;
            };

            let val = reader.read(field_name);
            writer.write(field_name, &val);
            self.state.save(field_name, sid, val);

            println!("#{sid}: {field_name:?}={val}");
        }

        let mut buf = vec![DecodeType::Single as u8];
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
                let mut writer = self.codec.writer();
                writer.write(&FieldName::DevSessionId, sid);

                for (field_name, _) in self.codec.schema().fields.iter() {
                    if let Some(val) = player_state.get(field_name) {
                        writer.write(field_name, val);
                    }
                }

                writer.buf()
            })
            .collect();

        let mut buf_sync = vec![DecodeType::Batch as u8, records.len() as u8];
        for record in records {
            buf_sync.push(record.len() as u8);
            buf_sync.extend(record);
        }
        buf_sync
    }

    pub fn has_state(&self) -> bool {
        !self.state.world.is_empty()
    }
}

struct State {
    world: WorldState,
}

impl State {
    pub fn new() -> Self {
        let world = WorldState::new();
        State { world }
    }

    pub fn save(&mut self, field_name: &FieldName, sid: u32, val: u32) {
        self.world
            .entry(sid)
            .or_default()
            .insert(field_name.clone(), val);
    }
}
