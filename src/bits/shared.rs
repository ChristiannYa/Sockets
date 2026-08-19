use crate::bits::FieldName;

#[derive(Debug)]
pub struct BufferProgress {
    /// Current byte index
    pub ind: usize,

    /// Current byte offset
    pub ofs: usize,

    /// Current byte remaining length
    pub rem_len: usize,

    /// Current byte overflow length
    pub ovf_len: usize,
}

impl BufferProgress {
    pub fn calc(bits_acc: usize, field_len: &usize) -> Self {
        let ind = bits_acc / 8;
        let ofs = bits_acc % 8;
        let rem_len = 8 - ofs;
        let ovf_len = field_len.saturating_sub(rem_len);
        BufferProgress { ind, ofs, rem_len, ovf_len }
    }
}

/// Returns the (index, length) tuple
pub fn field_meta_or_panic(
    fields: &[(FieldName, usize)],
    field_name: &FieldName,
) -> (usize, usize) {
    fields
        .iter()
        .enumerate()
        .find(|(_, (field_iter_name, _))| *field_iter_name == *field_name)
        .map(|(ind, (_, field_iter_len))| (ind, *field_iter_len))
        .unwrap_or_else(|| panic!("Field '{:?}' not found", field_name))
}
