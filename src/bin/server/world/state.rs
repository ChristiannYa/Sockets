use bitp::pkt::FieldName;
use std::collections::HashMap;

pub struct WorldState {
    pub world: HashMap<u32, HashMap<FieldName, u32>>,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            world: HashMap::new(),
        }
    }

    pub fn save(&mut self, field_name: &FieldName, sid: u32, val: u32) {
        self.world
            .entry(sid)
            .or_default()
            .insert(field_name.clone(), val);
    }
}
