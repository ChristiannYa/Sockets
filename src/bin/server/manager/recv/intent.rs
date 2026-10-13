use crate::manager::Manager;
use bitp::{
    pkt::FieldName,
    qua,
    sim::{Edges, Held, PlayerIntentCmd},
    util::Seq,
};

const INTENT_FIELDS: [FieldName; 6] = [
    FieldName::IntentSeq,
    FieldName::IntentMvX,
    FieldName::IntentMvZ,
    FieldName::IntentYaw,
    FieldName::IntentJump,
    FieldName::IntentCrouch,
];

pub struct IntentPkt<'i, 'c, 'w> {
    manager: &'i mut Manager<'c, 'w>,
    buf: &'i [u8],
    sid: u32,
}

impl<'i, 'c, 'w> IntentPkt<'i, 'c, 'w> {
    pub fn new(
        manager: &'i mut Manager<'c, 'w>,
        buf: &'i [u8],
        sid: u32,
    ) -> Self {
        IntentPkt { manager, buf, sid }
    }

    pub fn recv(&mut self) {
        let Some((seq, cmd)) = self.decode() else {
            return;
        };

        if let Some(player) = self.manager.world.player_ref(self.sid) {
            player.intent_queue.queue(seq, cmd);
        }
    }

    fn decode(&self) -> Option<(Seq, PlayerIntentCmd)> {
        let schema = self.manager.codec.schema();

        let payload_ceil_len = INTENT_FIELDS
            .iter()
            .map(|field_name| schema.info_of(field_name).len)
            .sum::<usize>()
            .div_ceil(8);
        if self.buf.len() < schema.mask_len() + payload_ceil_len {
            return None;
        }

        let mut reader = self.manager.codec.reader(self.buf);
        if !INTENT_FIELDS
            .iter()
            .all(|field_name| reader.isset(field_name))
        {
            return None;
        }

        let seq = Seq(reader.read(&FieldName::IntentSeq) as u8);

        let mv_x = qua::intent::MV.decode(reader.read(&FieldName::IntentMvX));
        let mv_z = qua::intent::MV.decode(reader.read(&FieldName::IntentMvZ));
        let yaw = qua::intent::YAW.decode(reader.read(&FieldName::IntentYaw));
        let jump_ct = reader.read(&FieldName::IntentJump) as u8;
        let crouch = reader.read(&FieldName::IntentCrouch) == 1;

        let held = Held { mv_x, mv_z, yaw, crouch };
        let edges = Edges { jump: jump_ct };
        let cmd = PlayerIntentCmd::new(held, edges);

        Some((seq, cmd))
    }
}
