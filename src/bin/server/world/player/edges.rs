//! Just for the sake of having an individual module to do some testing

use bitp::sim::PlayerIntentSimEdges;

/// Which counters differ between the edges.
///
/// Wraping needs no special case because any difference counts (7 -> 0
/// included)
pub fn diffs(
    a: PlayerIntentSimEdges<u8>,
    b: PlayerIntentSimEdges<u8>,
) -> PlayerIntentSimEdges<bool> {
    PlayerIntentSimEdges {
        jump: a.jump != b.jump,
        // shot: a.shot != b.shot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cts(jump: u8) -> PlayerIntentSimEdges<u8> {
        PlayerIntentSimEdges { jump }
    }

    #[test]
    fn changed_counter_fires() {
        assert!(diffs(cts(3), cts(2)).jump);
    }

    #[test]
    fn same_counter_does_not_fire() {
        assert!(!diffs(cts(2), cts(2)).jump);
    }

    #[test]
    fn wrap_fires() {
        // 3-bit counter: 7 is the max, the next press wraps to 0
        assert!(diffs(cts(0), cts(7)).jump);
    }

    #[test]
    fn skipped_counters_fire() {
        // 2 presses between received intents is still 1 jump
        assert!(diffs(cts(4), cts(2)).jump);
    }
}
