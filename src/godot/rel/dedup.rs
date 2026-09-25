use crate::rel::dedup::SeenPacketIds;
use godot::prelude::{Gd, GodotClass, godot_api};
use std::time::Instant;

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpDedup {
    ids: SeenPacketIds,
}

#[godot_api]
impl UdpDedup {
    #[func]
    fn create() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            ids: SeenPacketIds::new(),
        })
    }

    #[func]
    fn is_seen(&self, id: u8) -> bool {
        self.ids.is_seen(id)
    }

    #[func]
    fn mark_seen(&mut self, id: u8) {
        self.ids.mark_seen(id, Instant::now())
    }

    #[func]
    fn sweep(&mut self) {
        self.ids.sweep(Instant::now())
    }
}
