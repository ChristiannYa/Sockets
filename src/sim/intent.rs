#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Held {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub crouch: bool,
}

/// One-shot actions, with one field per event.
///
/// `T<bool>`: fired this tick.
/// `T<u8>`: a wrapping counter of presses.
///
/// See [PlayerIntent] for why both forms exist.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges<T> {
    pub jump: T,
    // pub shot: T,
}

/// A player's input for one tick: the state they're *in* (`held`) plus the
/// one-shot actions they performed (`edges`).
///
/// It is generic over how the edges are represented, so "held + edges" is
/// declared once and exists in two forms that wont't drift apart:
///
/// - `T<u8>`: what arrives from the client. Each edge is a
///   wrapping counter, which survives packet loss (see [Edges]).
/// - `T<bool>`: what [crate::sim::step::Step] consumes. Each edge is "fired
///   this tick", derived by comparing a counter with the previous one.
///
/// [PlayerIntent::held_only] exists only for `bool`, because a counter has no
/// "reset" (repeating it is what keeps the client's presses from being lost).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerIntent<T> {
    pub held: Held,
    pub edges: Edges<T>,
}

impl PlayerIntent<bool> {
    /// The intent to assume when the real one is missing: held state
    /// (movement, crouch, yaw) carry over, instant actions do not (e.g.
    /// jump, shot).
    pub fn held_only(self) -> PlayerIntent<bool> {
        PlayerIntent { held: self.held, edges: Edges::default() }
    }
}
