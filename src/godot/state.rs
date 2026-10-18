mod players;

use crate::{bits::PacketSchema, pkt::FIELD_LENGTHS};
use godot::prelude::{Gd, GodotClass, godot_api};
use players::PlayerStates;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpState {
    players: Gd<PlayerStates>,
}

#[godot_api]
impl UdpState {
    #[func]
    fn create(history_cap: i64) -> Gd<Self> {
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
        let cap = history_cap.max(2) as usize;
        let players = Gd::from_object(PlayerStates::new(schema, cap));
        Gd::from_init_fn(|_| Self { players })
    }

    #[func]
    fn players(&self) -> Gd<PlayerStates> { self.players.clone() }
}
