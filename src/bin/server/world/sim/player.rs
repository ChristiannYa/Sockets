use crate::world::player::PlayersHM;
use bitp::sim::{Collision, PlayerState, Step};

pub fn tick(hm: &mut PlayersHM) {
    step_all(hm);
    resolve_collisions(hm);
}

fn step_all(hm: &mut PlayersHM) {
    for player in hm.values_mut() {
        let sim_intent = player.intent_queue.next();
        Step::new(&mut player.state, &sim_intent).run();
    }
}

fn resolve_collisions(hm: &mut PlayersHM) {
    // Sorted by sid, because HashMap iteration order is random
    let mut states: Vec<(u32, &mut PlayerState)> = hm
        .iter_mut()
        .map(|(sid, p)| (*sid, &mut p.state))
        .collect();
    states.sort_unstable_by_key(|(sid, _)| *sid);

    Collision::new(
        states
            .into_iter()
            .map(|(_, p_s)| p_s)
            .collect(),
    )
    .resolve();
}
