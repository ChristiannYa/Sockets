mod prog;
pub mod reader;
pub mod writer;
use super::PacketSchema;
use crate::pkt::FieldName;

pub struct FieldInfo {
    pub ind: usize,
    pub len: usize,
}

#[repr(u8)]
pub enum DecodeType {
    Single,
    Batch,
}

impl PacketSchema {
    /// Returns (index, length)
    pub fn info_of(&self, field_name: &FieldName) -> FieldInfo {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == *field_name)
            .map(|(ind, (_, len))| FieldInfo { ind, len: *len })
            .unwrap_or_else(|| panic!("Field '{:?}' not found", field_name))
    }
}
