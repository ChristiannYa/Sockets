pub mod bits;
pub mod util;

#[cfg(feature = "godot")]
pub mod godot_bindings;

use crate::bits::FieldName;

pub static FIELD_LENGTHS: &[(bits::FieldName, usize)] = &[
    (FieldName::Health, 3),
    (FieldName::PlayerCount, 4),
    (FieldName::IsJumping, 1),
    (FieldName::IsCrouching, 1),
    (FieldName::IsFriendly, 1),
];
