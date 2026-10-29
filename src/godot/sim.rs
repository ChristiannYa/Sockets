mod predictor;

use crate::{
    qua::player::intent as pqua,
    sim::{
        PlayerIntentSim, PlayerIntentSimEdges, PlayerIntentSimHeld,
        PlayerState, Step,
    },
};
use godot::prelude::{Gd, GodotClass, Vector3, godot_api};

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct UdpLocalSim {
    /// [`None`] until [`LocalSim::seed`] is called
    state: Option<PlayerState>,

    /// Counters of the last intent sent, stepped or not. [`None`] for the
    /// first one, which seeds them (see [`PlayerIntentSimEdges::fired`])
    prev_edges: Option<PlayerIntentSimEdges<u8>>,
}

#[godot_api]
impl UdpLocalSim {
    #[func]
    pub fn create() -> Gd<Self> {
        Gd::from_init_fn(|_| Self { state: None, prev_edges: None })
    }

    /// Runs one [`Step`] on the intent as the server will see it: `x`, `z`,
    /// and `yaw` are rounded through the wire quantizers, so the client
    /// doesn't predict on values the server never receives.
    ///
    /// Call it once per intent sent, with the same values. It does not step
    /// until [`LocalSim::seed`] has been called.
    #[func]
    fn tick(
        &mut self,
        x: f32,
        z: f32,
        yaw: f32,
        is_crouching: bool,
        jump_ct: u8,
    ) {
        let counters = PlayerIntentSimEdges { jump: jump_ct };

        // Recorded before the seeded check on purpose. Say the seed lands
        // after intent #4 and the player jumps on #5 (counter 0 -> 1):
        //
        //   intent:        #1  #2  #3  #4 | #5
        //   jump counter:   0   0   0   0 |  1
        //   seeded:                       ^
        //
        //   server:        prev = 0 (seeded at #1) -> 1 != 0 -> fires
        //   skip:          prev = None at #5       -> "first intent" -> swallowed
        //   record always: prev = 0 (from #1..#4)  -> 1 != 0 -> fires
        //
        // so `prev` follows the same intent stream as the server's.
        let prev = self.prev_edges.replace(counters);

        let Some(state) = self.state.as_mut() else {
            return;
        };

        let intent = PlayerIntentSim {
            held: PlayerIntentSimHeld {
                mv_x: pqua::MV.decode(pqua::MV.encode(x)),
                mv_z: pqua::MV.decode(pqua::MV.encode(z)),
                yaw: pqua::YAW.decode(pqua::YAW.encode(yaw)),
                is_crouching,
            },
            edges: counters.fired(prev),
        };

        Step::new(state, &intent).run();
    }

    /// Starts the sim at the spawn defaults (standing, on the floor, at
    /// rest).
    #[func]
    fn seed(&mut self, x: f32, z: f32, yaw: f32) {
        let mut s = PlayerState::spawn(x, z);
        s.yaw = yaw;
        self.state = Some(s);
    }

    #[func]
    fn is_seeded(&self) -> bool { self.state.is_some() }

    /// Where the player is now, or the origin before the sim is seeded.
    #[func]
    fn pos(&self) -> Vector3 {
        self.state.as_ref().map_or(Vector3::ZERO, |s| {
            Vector3::new(s.pos[0], s.pos[1], s.pos[2])
        })
    }
}
