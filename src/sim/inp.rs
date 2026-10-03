/// State the player is *in*. Safe to repeat when an input is missing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Held {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub crouch: bool,
}

/// One-shot actions that happened on this tick. Never repeated.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub jump: bool,
    // pub shot: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInpSim {
    pub held: Held,
    pub edges: Edges,
}

impl PlayerInpSim {
    pub fn new(held: Held, edges: Edges) -> Self {
        PlayerInpSim { held, edges }
    }

    /// The input to assume when the real one is missing: held state (movement,
    /// crouch, yaw) carry over, instant actions do not (e.g. jump, shot) do not.
    pub fn held_only(self) -> PlayerInpSim {
        PlayerInpSim {
            held: self.held,
            edges: Edges::default(),
        }
    }
}
