use crate::{
    codec::{Codec, FieldsPack},
    world::RelTarget,
};
use bitp::{pkt::FieldName, rel::retx::PendingPacket};
use std::time::Instant;

pub struct WorldPacker<'w> {
    codec: &'w Codec,
}

impl<'w> WorldPacker<'w> {
    pub fn new(codec: &'w Codec) -> Self { WorldPacker { codec } }

    /// Returns a reliable packet with the [FieldName::DevIsNewPlayer] flag
    /// Information: [DecodeType::Single], [FieldName::DevSessionId] and
    /// [FieldName::DevIsNewPlayer]
    pub fn welcome_pkt(&self, rel_target: &mut RelTarget) -> Vec<u8> {
        let (pkt, _) =
            self.rel_pkting(&[(FieldName::DevIsNewPlayer, 1)], rel_target);
        pkt
    }

    fn rel_pkting(
        &self,
        fields: &[(FieldName, u32)],
        rel_target: &mut RelTarget,
    ) -> (Vec<u8>, u8) {
        let id = self.codec.next_pkt_id();
        let pkt = self.codec.headful_pack(FieldsPack {
            id: Some(id),
            fields,
            sid: rel_target.sid,
        });

        rel_target.pending_pkts.insert(
            (id, *rel_target.skt_src),
            PendingPacket::new(id, pkt.to_vec(), Instant::now()),
        );

        (pkt, id)
    }
}
