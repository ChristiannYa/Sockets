use std::collections::HashMap;

use bitp::bits::FieldName;

pub type ClientStates = HashMap<u32, HashMap<FieldName, u32>>;

pub struct State {
    clis: ClientStates,
}

impl State {
    pub fn new() -> Self {
        let clis = ClientStates::new();
        State { clis }
    }

    pub fn save_cli(&mut self, field_name: &FieldName, sid: u32, val: u32) {
        self.clis
            .entry(sid)
            .or_default()
            .insert(field_name.clone(), val);
    }

    pub fn clis(&self) -> &ClientStates {
        &self.clis
    }
}
