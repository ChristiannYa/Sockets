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
        BufferProgress {
            ind,
            ofs,
            rem_len,
            ovf_len,
        }
    }
}
