//! Smooth movement of remote players.
//!
//! The server sends every player's pose in snapshots 60 times a second.
//! Drawing each pose as it arrives would look jittery, so poses are kept with
//! their time and the game asks for the pose at a moment slightly in the past,
//! blended between the two snapshots around that moment.
//!
//! A record can carry only some of the pose fields, so each player's track
//! merges them with what is already known and decodes a pose once every
//! field is known.

mod raw;
mod sample;
mod track;

pub use raw::RawPose;
pub use sample::{POSE_STRIDE, PoseSample};
pub use track::PoseTrack;
