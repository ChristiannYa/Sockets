mod reader;
mod shared;
mod writer;

pub use reader::BitReader;
pub use writer::BitWriter;

pub struct FieldInfo {
    pub ind: usize,
    pub len: usize,
}

#[derive(Clone)]
pub struct PacketSchema<'a> {
    pub fields: &'a [(FieldName, usize)],
}

impl<'a> PacketSchema<'a> {
    pub fn build(fields: &'a [(FieldName, usize)]) -> Result<Self, String> {
        for (field_name, field_len) in fields {
            let reason =
                |msg: &str| -> String { format!("Field '{:?}' is invalid: {msg}", field_name) };

            if *field_len == 0 {
                return Err(reason("Length cannot be 0"));
            }

            if fields
                .iter()
                .filter(|(field_iter_name, _)| field_iter_name == field_name)
                .count()
                > 1
            {
                return Err(reason("Already exists"));
            }
        }

        Ok(PacketSchema { fields })
    }

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

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum FieldName {
    SessionId,
    Sequence,
    Ping,
    IsNewPlayer,
    IsJumping,
    IsPremium,
    IsFriendly,
    IsLucky,
    IsCrouching,
    LocationX,
    LocationZ,
    RocketsCount,
    PlayerCount,
    KeysCount,
    Health,
    Level,
    Mana,
    Time24,
    Exp,
    Age,
    Year,
    Boss,
}
