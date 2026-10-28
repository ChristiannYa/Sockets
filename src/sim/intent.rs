#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerIntentSimHeld {
    pub mv_x: f32,
    pub mv_z: f32,
    pub yaw: f32,
    pub is_crouching: bool,
}

/// One-shot actions, with one field per event.
///
/// `T<bool>`: fired this tick.
/// `T<u8>`: a wrapping counter of presses.
///
/// See [PlayerIntentSim] for why both forms exist.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerIntentSimEdges<T> {
    pub jump: T,
    // pub melee: T,
}

impl PlayerIntentSimEdges<u8> {
    /// Which counters differ between these edges (the current ones) and
    /// `prev` (the ones simulated before).
    ///
    /// Wraping needs no special case because any difference counts (7 -> 0
    /// included)
    pub fn diffs(
        &self,
        prev: PlayerIntentSimEdges<u8>,
    ) -> PlayerIntentSimEdges<bool> {
        PlayerIntentSimEdges {
            jump: self.jump != prev.jump,
            // melee: self.melee != prev.melee
        }
    }

    /// Edges fired by `self`, given the counters of the intent simulated
    /// right before it (`prev`). With no previous counters (the first
    /// intent) nothing fires, because the first intent only seeds them.
    ///
    /// The server and the client's replay both call this, but their `prev`
    /// can differ when an intent is lost. The server compares against the
    /// last intent that arrived, so a lost intent's counter change fires on
    /// the next one. The client has every intent it sent, so it fires on
    /// the one where the counter changed. The server's jump comes one tick
    /// later than the client predicted, until the next snapshot corrects
    /// it.
    ///
    /// ```text
    /// intent (client seq):   #4    #5     #6
    /// jump counter:          1     2      2
    /// arrived at server:     yes   LOST   yes
    ///
    /// server fires on:                    #6   (2 != 1, the counter of #4)
    /// client replay fires on:      #5          (2 != 1, the counter of #4)
    /// ```
    pub fn fired(
        &self,
        prev: Option<PlayerIntentSimEdges<u8>>,
    ) -> PlayerIntentSimEdges<bool> {
        match prev {
            Some(prev) => self.diffs(prev),
            None => PlayerIntentSimEdges::default(),
        }
    }
}

/// A player's input for one tick: the state they're *in* (`held`) plus the
/// one-shot actions they performed (`edges`).
///
/// It is generic over how the edges are represented, so "held + edges" is
/// declared once and exists in two forms that won't drift apart:
///
/// - `T<u8>`: what arrives from the client. Each edge is a
///   wrapping counter, which survives packet loss
///   (see [PlayerIntentSimEdges]).
/// - `T<bool>`: what [crate::sim::step::Step] consumes. Each edge is "fired
///   this tick", derived by comparing a counter with the previous one.
///
/// [PlayerIntentSim::held_only] exists only for `bool`, because a counter has
/// no "reset" (repeating it is what keeps the client's presses from being
/// lost).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerIntentSim<T> {
    pub held: PlayerIntentSimHeld,
    pub edges: PlayerIntentSimEdges<T>,
}

impl PlayerIntentSim<bool> {
    /// The intent to assume when the real one is missing: held state
    /// (movement, crouch, yaw) carry over, instant actions do not (e.g.
    /// jump, shot).
    pub fn held_only(self) -> PlayerIntentSim<bool> {
        PlayerIntentSim {
            held: self.held,
            edges: PlayerIntentSimEdges::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edg(jump: u8) -> PlayerIntentSimEdges<u8> {
        PlayerIntentSimEdges { jump }
    }

    #[test]
    fn changed_counter_fires() {
        assert!(edg(3).diffs(edg(2)).jump);
    }

    #[test]
    fn same_counter_does_not_fire() {
        assert!(!edg(2).diffs(edg(2)).jump);
    }

    #[test]
    fn wrap_fires() {
        // 3-bit counter: 7 is the max, the next press wraps to 0
        assert!(edg(0).diffs(edg(7)).jump);
    }

    #[test]
    fn skipped_counters_fire() {
        // 2 presses between received intents is still 1 jump
        assert!(edg(4).diffs(edg(2)).jump);
    }
}
