/// State the player is *in*.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Held {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub crouch: bool,
}

/// One-shot actions.
///
/// `Edges<bool>` is the sim form ("applied this tick").
/// `Edges<u8>` is the wire form (a wrapping counter per event).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges<T> {
    pub jump: T,
    // pub shot: T,
}

/// What the client asked for, before the server turns it into a sim intent.
#[derive(Clone, Copy, Debug)]
pub struct PlayerIntentCmd {
    pub held: Held,
    pub edge_cts: Edges<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerIntentSim {
    pub held: Held,
    pub edges: Edges<bool>,
}

impl PlayerIntentSim {
    pub fn new(held: Held, edges: Edges<bool>) -> Self {
        PlayerIntentSim { held, edges }
    }

    /// The intent to assume when the real one is missing: held state
    /// (movement, crouch, yaw) carry over, instant actions do not (e.g.
    /// jump, shot).
    pub fn held_only(self) -> PlayerIntentSim {
        PlayerIntentSim { held: self.held, edges: Edges::default() }
    }
}
