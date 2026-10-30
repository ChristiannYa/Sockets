use crate::{
    bits::{BitWriter, PacketSchema},
    pkt::{FieldName, PacketKind},
    qua,
    sim::PlayerIntentSim,
};

/// No sid: Obtained internally using the client's address.
/// No packet id: It is not reliable.
#[rustfmt::skip]
pub fn encode(
    schema: &PacketSchema,
    seq: u8,
    intent: &PlayerIntentSim<u8>,
) -> Vec<u8> {
    let mut writer = BitWriter::new(schema);

    writer.write(&FieldName::IntentSeq, &(seq as u32));
    writer.write(&FieldName::IntentMvX, &qua::player::intent::MV.encode(intent.held.x));
    writer.write(&FieldName::IntentMvZ, &qua::player::intent::MV.encode(intent.held.z));
    writer.write(&FieldName::IntentYaw, &qua::player::intent::YAW.encode(intent.held.yaw));
    writer.write(&FieldName::IntentJump, &(intent.edges.jump as u32));
    writer.write(&FieldName::IntentCrouch, &(intent.held.is_crouching as u32));

    let mut pkt = vec![PacketKind::Intent as u8];
    pkt.extend(writer.buf());
    pkt
}
