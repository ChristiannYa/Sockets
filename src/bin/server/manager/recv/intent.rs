use std::net::SocketAddr;

use crate::manager::Manager;
use bitp::{
    pkt::{FieldName, Seq},
    qua,
    sim::{PlayerIntentSim, PlayerIntentSimEdges, PlayerIntentSimHeld},
};

const INTENT_FIELDS: [FieldName; 6] = [
    FieldName::IntentSeq,
    FieldName::IntentMvX,
    FieldName::IntentMvZ,
    FieldName::IntentYaw,
    FieldName::IntentJump,
    FieldName::IntentCrouch,
];

pub fn recv(manager: &mut Manager, skt_src: SocketAddr, buf: &[u8]) {
    let Some(sid) = manager.sess.id_of(&skt_src) else {
        return;
    };

    let Some(buf) = buf.get(1..) else {
        return;
    };

    let Some((seq, sim)) = decode(manager, buf) else {
        return;
    };

    if let Some(player) = manager.world.player_mut_ref(sid) {
        player.intent_queue.queue(seq, sim);
    }
}

#[rustfmt::skip]
fn decode(
    manager: &Manager,
    buf: &[u8],
) -> Option<(Seq, PlayerIntentSim<u8>)> {
    let schema = manager.codec.schema();

    let payload_ceil_len = INTENT_FIELDS
        .iter()
        .map(|field_name| field_name.blen())
        .sum::<usize>()
        .div_ceil(8);
    if buf.len() < schema.mask_len() + payload_ceil_len {
        return None;
    }

    let mut reader = manager.codec.reader(buf);
    if !INTENT_FIELDS
        .iter()
        .all(|field_name| reader.isset(field_name))
    {
        return None;
    }

    let seq = Seq(reader.read(&FieldName::IntentSeq) as u8);

    let x = qua::player::intent::MV.decode(reader.read(&FieldName::IntentMvX));
    let z = qua::player::intent::MV.decode(reader.read(&FieldName::IntentMvZ));
    let yaw = qua::player::intent::YAW.decode(reader.read(&FieldName::IntentYaw));
    let jump_ct = reader.read(&FieldName::IntentJump) as u8;
    let is_crouching = reader.read(&FieldName::IntentCrouch) == 1;

    let held =
        PlayerIntentSimHeld { x, z, yaw, is_crouching };
    let edges = PlayerIntentSimEdges { jump: jump_ct };
    let sim = PlayerIntentSim::<u8> { held, edges };

    Some((seq, sim))
}
