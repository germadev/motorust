//! Loop Island: a lap inspired by Green Hill, the first zone of Sonic the
//! Hedgehog. A roller coaster of a road, clockwise round an island of green
//! hills under a blue sky, with the sea all around.
//!
//! 1. The start straight runs north between palm trees and sunflowers, with
//!    a crest where the bike takes off just past the line, and over a broad
//!    top where the branch to the loop parts from it.
//! 2. A long right-hander climbs round the island's own loop, and S-bends
//!    roll on east over the hills. On the climb to the top, a branch forks
//!    off to the right, sweeps inside the corner and goes round the loop,
//!    upside down at the top, before it joins the lap again on the S-bends.
//! 3. The road climbs into a tunnel through a checkerboard ridge and comes
//!    out by a waterfall, high above the sea, into a long banked right-hander.
//! 4. Down past a second waterfall, the road turns right and comes down
//!    nearly to the sea, round a round bay with an islet in it, the sea
//!    running in where the road crossed it on a bridge, and back up.
//! 5. Rolling S-bends with totem poles lead down to the south shore: a
//!    beach road with palms, over a lagoon on a wooden bridge narrower than
//!    the road, back to the start.
//!
//! The circuit is written in RON, `assets/circuits/loop_island.ron`, its
//! mountains too; here is a test of its branch.

#[cfg(test)]
mod tests {
    use crate::game::CIRCUITS;
    use motorust_engine::track::Track;

    /// Where the branch with the loop forks off the lap, and where the
    /// start straight ends (m from the finish line).
    const FORK: f32 = 110.0;
    const START_END: f32 = 290.0;

    #[test]
    fn the_branch_forks_past_the_jump_over_a_broad_top_and_turns_gently() {
        crate::game::register();
        let circuit = CIRCUITS.iter().find(|c| c.name == "Loop Island").unwrap();
        let track = Track::new(&circuit.layout);
        let crest = |along: f32| {
            let index = track.index_at(along);
            track.crest(&motorust_engine::track::Nearest {
                point: track.centerline()[index],
                distance: 0.0,
                index,
                t: 0.0,
                lateral: 0.0,
                road: 0,
            })
        };
        // The bike takes off just past the line, and lands well before the
        // branch forks off.
        let jump = (0..FORK as usize)
            .step_by(2)
            .max_by(|&a, &b| crest(a as f32).total_cmp(&crest(b as f32)))
            .unwrap() as f32;
        assert!(jump < FORK - 40.0, "the jump is at {jump} m");
        // Where the branch parts from the lap, over the top of the start
        // straight, no bike takes off, even at its top speed: one flying
        // there landed in the branch's bend. (Against the real gravity: a
        // bike takes off against twice it, so with room to spare.)
        let top = motorust_engine::bike::BikeTuning::default().max_speed;
        for along in (FORK as usize..START_END as usize).step_by(2) {
            let crest = crest(along as f32);
            assert!(
                crest * top * top < 9.81,
                "a crest of {:.0} m at {along} m",
                1.0 / crest
            );
        }
        // The road to the loop turns off the lap in a long, easy curve, and
        // the road back is no tighter than the lap where it joins it.
        let looped = track.loops()[0];
        for (road, tightest) in [(looped.approach, 100.0), (looped.exit, 75.0)] {
            let road = track.road(road);
            for i in 1..road.sample_count() - 1 {
                let radius = 1.0 / road.curvature(i).abs();
                assert!(radius > tightest, "{radius} m at {i}");
            }
        }
    }
}
