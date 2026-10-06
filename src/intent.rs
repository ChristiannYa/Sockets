use crate::{
    bits::{BitWriter, PacketSchema},
    pkt::{FieldName, PacketKind},
    quantize::intent::{MV, YAW},
    sim::PlayerIntentSim,
};

/// No sid: Obtained internally using the client's address.
/// No packet id: It is not reliable.
pub fn encode(
    schema: &PacketSchema,
    seq: u8,
    intent: &PlayerIntentSim,
) -> Vec<u8> {
    let mut writer = BitWriter::new(schema);
    writer.write(&FieldName::IntentSeq, &(seq as u32));
    writer.write(&FieldName::IntentMvX, &MV.encode(intent.held.mv_x));
    writer.write(&FieldName::IntentMvZ, &MV.encode(intent.held.mv_z));
    writer.write(&FieldName::IntentYaw, &YAW.encode(intent.held.yaw));
    writer.write(&FieldName::IntentCrouch, &(intent.held.crouch as u32));
    writer.write(&FieldName::IntentJump, &(intent.edges.jump as u32));

    let mut pkt = vec![PacketKind::Intent as u8];
    pkt.extend(writer.buf());
    pkt
}
