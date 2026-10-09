#[repr(u8)]
pub enum PacketKind {
    Data,
    Ack,
    Intent,
    Snapshot,
}

impl TryFrom<u8> for PacketKind {
    type Error = ();

    /// Turn raw byte from a packet into [PacketType]
    fn try_from(buf: u8) -> Result<Self, ()> {
        match buf {
            0 => Ok(Self::Data),
            1 => Ok(Self::Ack),
            2 => Ok(Self::Intent),
            _ => Err(()),
        }
    }
}
