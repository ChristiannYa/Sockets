use super::FieldName;

/// @TODO
/// Replace with inline function
pub static FIELD_LENGTHS: &[(FieldName, usize)] = &[
    (FieldName::DevSid, 6),
    (FieldName::DevSeq, 8),
    (FieldName::DevSimedSeq, 8),
    (FieldName::DevPing, 1),
    (FieldName::DevPktId, 8),
    (FieldName::DevImNew, 1),
    (FieldName::IntentSeq, 8),
    (FieldName::IntentMvX, 6),
    (FieldName::IntentMvZ, 6),
    (FieldName::IntentYaw, 10),
    (FieldName::IntentJump, 3),
    (FieldName::IntentCrouch, 1),
    (FieldName::LocationX, 11),
    (FieldName::LocationZ, 11),
    (FieldName::LocationY, 8),
    (FieldName::Yaw, 10),
    (FieldName::ColorH, 6),
    (FieldName::ColorS, 3),
    (FieldName::ColorV, 3),
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::IsJumping, 1),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
    (FieldName::VelY, 8),
];

impl FieldName {
    #[inline(always)]
    pub const fn blen(&self) -> usize {
        match self {
            FieldName::DevSid => 6,
            FieldName::DevSeq => 8,
            FieldName::DevSimedSeq => 8,
            FieldName::DevPing => 1,
            FieldName::DevPktId => 8,
            FieldName::DevImNew => 1,
            FieldName::IntentSeq => 8,
            FieldName::IntentMvX => 6,
            FieldName::IntentMvZ => 6,
            FieldName::IntentYaw => 10,
            FieldName::IntentJump => 3,
            FieldName::IntentCrouch => 1,
            FieldName::LocationX => 11,
            FieldName::LocationZ => 11,
            FieldName::LocationY => 8,
            FieldName::Yaw => 10,
            FieldName::VelY => 8,
            FieldName::ColorH => 6,
            FieldName::ColorS => 3,
            FieldName::ColorV => 3,
            FieldName::Health => 3,
            FieldName::PlayerCount => 4,
            FieldName::IsJumping => 1,
            FieldName::IsCrouching => 1,
            FieldName::IsFriendly => 1,
            _ => 1,
        }
    }
}
