use std::collections::HashMap;

use bitp::{
    FIELD_LENGTHS, PacketKind,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
    hsv_codec, loc_codec,
};

pub mod logic;

type WorldState = HashMap<u32, HashMap<FieldName, u32>>;

pub struct World<'a> {
    schema: PacketSchema<'a>,
    state: State,
}

impl<'a> World<'a> {
    pub fn new() -> Self {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        World {
            schema,
            state: State::new(),
        }
    }

    /// Returns a buffer with information indicating that the player is new.
    /// Information: [PacketKind]'s `Single`, [FieldName]'s `SessionId` and `IsNewPlayer`
    pub fn welcome_buf(&self, sid: u32) -> Vec<u8> {
        self.pack(&[(FieldName::DevIsNewPlayer, 1)], sid)
    }

    /// Returns a buffer with the X and Z location of the player
    pub fn spawn_loc_buf(&mut self, sid: u32) -> Vec<u8> {
        let spawn_pt = logic::spawn::player_spawn_pt();
        let loc_codec = loc_codec(&self.schema);
        let fields = [
            (FieldName::LocationX, loc_codec.x.encode(spawn_pt.x)),
            (FieldName::LocationZ, loc_codec.z.encode(spawn_pt.z)),
        ];

        for (field_name, val) in &fields {
            self.state.save(field_name, sid, *val);
        }

        self.pack(&fields, sid)
    }

    /// Returns a buffer with spawn information
    /// - X and Z location of the player
    /// - HSV color of the player
    pub fn player_spawn_buf(&mut self, sid: u32) -> Vec<u8> {
        let (spawn_pt, spawn_pt_codec) = (logic::spawn::player_spawn_pt(), loc_codec(&self.schema));
        let (hsv, hsv_codec) = (logic::color::hsv(), hsv_codec(&self.schema));

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

        self.pack(&fields, sid)
    }

    /// Pack arbitrary values into a fresh buffer
    fn pack(&self, fields: &[(FieldName, u32)], sid: u32) -> Vec<u8> {
        let mut writer = BitWriter::new(&self.schema);
        writer.write(&FieldName::DevSessionId, &sid);

        for (name, val) in fields {
            writer.write(name, val);
        }

        let mut buf = vec![PacketKind::Single as u8];
        buf.extend(writer.buf());
        buf
    }

    /// Reads every field out of `buf`, echoing each one into a new packet seeded
    /// with `sid` and `seq` for the broadcast.
    /// Returns the broadcast buffer.
    pub fn process(&mut self, buf: &[u8], sid: u32, seq: u8) -> Vec<u8> {
        let mut reader = BitReader::new(&self.schema, buf);
        let mut writer = BitWriter::new(&self.schema);

        // Seed packet
        writer.write(&FieldName::DevSessionId, &sid);
        writer.write(&FieldName::DevSequence, &(seq as u32));

        // Write and save fields
        for (field_name, _) in self.schema.fields.iter() {
            if !reader.isset(field_name) {
                continue;
            };

            let val = reader.read(field_name);
            writer.write(field_name, &val);
            self.state.save(field_name, sid, val);

            println!("#{sid}: {field_name:?}={val}");
        }

        let mut buf = vec![PacketKind::Single as u8];
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
                let mut writer = BitWriter::new(&self.schema);
                writer.write(&FieldName::DevSessionId, sid);

                for (field_name, _) in self.schema.fields.iter() {
                    if let Some(val) = player_state.get(field_name) {
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

    pub fn is_empty(&self) -> bool {
        self.state.world.is_empty()
    }

    pub fn schema_min_len(&self) -> usize {
        self.schema.fields.len().div_ceil(8)
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
