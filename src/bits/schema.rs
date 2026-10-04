use crate::pkt::FieldName;

pub struct FieldInfo {
    pub ind: usize,
    pub len: usize,
}

#[derive(Clone)]
pub struct PacketSchema {
    pub fields: &'static [(FieldName, usize)],
}

impl PacketSchema {
    pub fn build(fields: &'static [(FieldName, usize)]) -> Result<Self, String> {
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

    pub fn mask_len(&self) -> usize {
        self.fields.len().div_ceil(8)
    }

    /// @TODO: Handle the case where it does not protect when a client sends
    /// ```
    /// [0b00000100, 0b00000000] // mask only, payload is empty
    /// ```
    pub fn mask_fits(&self, buf: &[u8]) -> bool {
        buf.len() >= self.mask_len()
    }
}
