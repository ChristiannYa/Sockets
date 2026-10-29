mod intent_queue;

use super::{player::intent_queue::PlayerIntentQueue, sim};
use bitp::{
    pkt::FieldName,
    qua,
    sim::PlayerState,
    util::{Hsv, rand_disk_pt_xz},
};
use std::collections::HashMap;

pub type PlayersHM = HashMap<u32, Player>;

#[derive(Default)]
pub struct Players {
    hm: PlayersHM,
}

impl Players {
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &Player)> {
        self.hm.iter()
    }

    pub fn by_sid_mut_ref(&mut self, sid: u32) -> Option<&mut Player> {
        self.hm.get_mut(&sid)
    }

    pub fn insert_new(&mut self, player: Player, sid: u32) {
        self.hm.insert(sid, player);
    }

    pub fn sim_tick(&mut self) { sim::players::tick(&mut self.hm); }
}

pub struct Player {
    pub state: PlayerState,
    pub intent_queue: PlayerIntentQueue,
    pub color: Hsv,
}

impl Player {
    pub fn new(x: f32, z: f32, color: Hsv) -> Self {
        Player {
            state: PlayerState::spawn(x, z),
            intent_queue: PlayerIntentQueue::new(),
            color,
        }
    }

    /// Returns a new player along with essential spawn information fields
    /// encoded
    pub fn spawn() -> (Player, [(FieldName, u32); 5]) {
        let (hsv, hsv_codec) = (Hsv::rand(), qua::hsv());

        let ((spawn_x, spawn_z), spawn_codec) =
            (rand_disk_pt_xz(10.0), qua::player::loc());

        let (enc_x, enc_z) =
            (spawn_codec.x.encode(spawn_x), spawn_codec.z.encode(spawn_z));

        let fields = [
            (FieldName::LocationX, enc_x),
            (FieldName::LocationZ, enc_z),
            (FieldName::ColorH, hsv_codec.h.encode(hsv.h)),
            (FieldName::ColorS, hsv_codec.s.encode(hsv.s)),
            (FieldName::ColorV, hsv_codec.v.encode(hsv.v)),
        ];

        let player = Player::new(
            spawn_codec.x.decode(enc_x),
            spawn_codec.z.decode(enc_z),
            Hsv::new(hsv.h, hsv.s, hsv.v),
        );

        (player, fields)
    }
}
