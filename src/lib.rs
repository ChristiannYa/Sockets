pub mod bits;
pub mod util;

use crate::bits::FieldName;

pub static FIELD_LENGTHS: &[(bits::FieldName, usize)] = &[
    (FieldName::IsJumping, 1),
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::Level, 4),
    (FieldName::Mana, 4),
    (FieldName::RocketsCount, 3),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
    (FieldName::KeysCount, 3),
];
