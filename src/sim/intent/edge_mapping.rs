use super::Edges;

/// Turns counters into one-tick edges.
#[derive(Clone, Debug, Default)]
pub struct PlayerEdgeMapper {
    latest: Option<Edges<u8>>,
}

impl PlayerEdgeMapper {
    /// The first counters ever seen only seed `latest`, so a joiner whose
    /// counters aren't 0 can't apply phantom events.
    pub fn to_sim(&mut self, new: Edges<u8>) -> Edges<bool> {
        let updated = match self.latest {
            Some(latest) => Edges {
                jump: latest.jump != new.jump,
                // shot: latest.shot != new.shot
            },
            None => Edges::default(),
        };
        self.latest = Some(new);
        updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cts(jump: u8) -> Edges<u8> { Edges { jump } }

    #[test]
    fn first_counters_only_seed() {
        let mut t = PlayerEdgeMapper::default();

        // Not 0 on purpose because a joiner's counter can start anywhere
        assert!(!t.to_sim(cts(5)).jump);
        assert!(!t.to_sim(cts(5)).jump);
    }

    #[test]
    fn change_updates_once() {
        let mut t = PlayerEdgeMapper::default();
        t.to_sim(cts(2));

        assert!(t.to_sim(cts(3)).jump);
        assert!(!t.to_sim(cts(3)).jump);
    }

    #[test]
    fn repeat_does_not_update() {
        let mut t = PlayerEdgeMapper::default();
        t.to_sim(cts(2));

        for _ in 0..5 {
            assert!(!t.to_sim(cts(2)).jump);
        }
    }

    #[test]
    fn wrap_updates() {
        let mut t = PlayerEdgeMapper::default();
        t.to_sim(cts(7));

        assert!(t.to_sim(cts(0)).jump);
    }

    #[test]
    fn skipped_counters_update_once() {
        // 2 presses between received intents is still 1 jump
        let mut t = PlayerEdgeMapper::default();
        t.to_sim(cts(2));

        assert!(t.to_sim(cts(4)).jump);
        assert!(!t.to_sim(cts(4)).jump);
    }
}
