use crate::bits::{self, FieldName};

pub static SCHEMA_FIELDS: &[(bits::FieldName, usize)] = &[
    (FieldName::DevSessionId, 6),
    (FieldName::DevSequence, 8),
    (FieldName::DevPacketId, 8),
    (FieldName::DevPing, 1),
    (FieldName::LocationX, 8),
    (FieldName::LocationZ, 8),
    (FieldName::ColorH, 6),
    (FieldName::ColorS, 3),
    (FieldName::ColorV, 3),
    (FieldName::DevIsNewPlayer, 1),
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::IsJumping, 1),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
];
