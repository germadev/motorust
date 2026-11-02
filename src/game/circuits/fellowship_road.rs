//! Fellowship Road: the journey of *The Fellowship of the Ring*, its
//! stretches in the order of the film, from the Shire to the Argonath and,
//! past the finish, home again (*There and Back Again*). It crosses the
//! Misty Mountains twice, in a figure of eight: first over them, by the pass
//! of Caradhras, and then, as the Fellowship turned back from the snow,
//! through them, by the mines of Moria, under the pass.
//!
//! 1. Hobbiton: the start straight, north through the wood, over a crest,
//!    a long sweep to the right down to the Water and low over it, a
//!    sweep to the left round the village's hill, and one to the right
//!    past Bag End's hill.
//! 2. The Shire: a sweep to the right and the left over the bridge on the
//!    Brandywine, and through its fields.
//! 3. Bree: through its gate, right and left through the town.
//! 4. Weathertop: up its flank at up to 12 %, round to the left and the
//!    right, over its top on a straight with a crest where the bike takes
//!    off, 51 m up, and down at up to 12 %.
//! 5. The Trollshaws: down in esses to the Ford of Bruinen, low over the
//!    water.
//! 6. Rivendell: south along the valley under its cliffs, where the road
//!    splits in two: the lap snakes along it and a way to the left swings
//!    out over a hump.
//! 7. Caradhras: up in bends at up to 12 % to the pass, 49 m up, right over
//!    the mines.
//! 8. Down round the mountain: on south, round to the left, east, north and
//!    back west, to the Doors of Durin.
//! 9. Moria: through the Doors of Durin, open, where the road narrows; west
//!    through the mountain, under the pass, 38 m beneath it,
//!    and on in a sweep to the left and the right, down into it and up
//!    again.
//! 10. Lothlórien: where the road splits three ways, the lap snaking
//!     between a way to the left over a hump and one to the right through a
//!     dip, all of them up and down, high over the land.
//! 11. The Anduin: down to the river, along its gorge in esses, and west
//!     down its middle on a causeway, narrower than the road, in a sweep to
//!     the right and the left between the two kings of the Argonath.
//! 12. Amon Hen: past the lake, round to the right over the shoulder of the
//!     hill, and down onto the start straight, back in the Shire.
//!
//! The circuit is written in RON, `assets/circuits/fellowship_road.ron`,
//! its water and its mountains too; here are tests of it.

mod tests {
    use bevy::prelude::*;

    /// How high the circuit's mountains stand at `point`, as its file has
    /// them.
    fn mountains(point: Vec2) -> f32 {
        fellowship_road().scenery.mountains.height(point)
    }
    use crate::game::CIRCUITS;
    use motorust_engine::circuits::Circuit;
    use motorust_engine::track::Track;
    use motorust_engine::world::scenery::{Coast, Span, span};

    /// Where things are, as in the circuit's file (m from the finish line).
    const TROLLSHAWS: Span = span(1959.1, 2300.1);
    const FORD: Span = span(2300.1, 2360.1);
    const CARADHRAS: Span = span(2819.1, 3219.1);
    const PASS: Span = span(3155.1, 3219.1);
    const MORIA: Span = span(3790.1, 4133.1);
    const GORGE: Span = span(4587.1, 4900.1);
    const CAUSEWAY: Span = span(4900.1, 5235.1);
    const RIVENDELL_SPLIT: (f32, f32) = (2364.1, 2714.1);
    /// Caradhras over the mines: its middle and its radius (m).
    const CARADHRAS_PEAK: Vec2 = Vec2::new(1595.0, 385.0);
    const CARADHRAS_RADIUS: f32 = 280.0;
    /// Where the two kings of the Argonath stand, either side of the
    /// causeway.
    const KINGS: [Vec2; 2] = [Vec2::new(426.5, 394.4), Vec2::new(441.0, 350.7)];
    /// Weathertop, from its foot, and its top: a straight over a crest.
    const WEATHERTOP: Span = span(1299.1, TROLLSHAWS.from);
    const SUMMIT: Span = span(1683.1, 1863.1);

    fn fellowship_road() -> &'static Circuit {
        CIRCUITS
            .iter()
            .find(|c| c.name_en == "Fellowship Road")
            .unwrap()
    }

    #[test]
    fn the_lap_goes_over_caradhras_and_then_through_moria_beneath() {
        crate::game::register();
        let track = Track::new(&fellowship_road().layout);
        let line = track.centerline();
        // Through the mines at the end, over the pass first: crossing well
        // above.
        let under = track.index_at(MORIA.from)..track.index_at(MORIA.to);
        let (gap, over, under) = (track.index_at(PASS.from)..track.index_at(PASS.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the pass and the mines have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 35.0, "{above} m above");
        // Under the mountain's top.
        assert!(line[under].distance(CARADHRAS_PEAK) < 0.3 * CARADHRAS_RADIUS);
        assert!(
            fellowship_road()
                .scenery
                .massifs
                .iter()
                .any(|m| m.center == CARADHRAS_PEAK && m.radius == CARADHRAS_RADIUS)
        );
    }

    #[test]
    fn the_hills_are_steep_and_the_road_climbs_and_drops_a_lot() {
        crate::game::register();
        let track = Track::new(&fellowship_road().layout);
        let step = track.step_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let grade =
            |index: usize| (track.elevations()[index + 1] - track.elevations()[index]) / step;
        let steepest = |stretch: Span| {
            let indices = track.index_at(stretch.from)..track.index_at(stretch.to);
            let up = indices.clone().map(grade).fold(0.0, f32::max);
            let down = indices.map(grade).fold(0.0, f32::min);
            (up, -down)
        };
        // Up Weathertop, 40 m and more, and down it to the ford, at nearly
        // 12 %.
        assert!(height(SUMMIT.from) - height(WEATHERTOP.from) > 35.0);
        assert!(height(SUMMIT.to) - height(FORD.from) > 40.0);
        assert!(steepest(WEATHERTOP).0 > 0.115);
        assert!(steepest(span(SUMMIT.to, FORD.from)).1 > 0.115);
        // Up from Rivendell over Caradhras, 30 m and more, at more than
        // 11 %, and down round it to the mines.
        let pass = (track.index_at(PASS.from)..track.index_at(PASS.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(
            pass - height(RIVENDELL_SPLIT.1) > 30.0,
            "{pass} m at the top"
        );
        assert!(steepest(CARADHRAS).0 > 0.11);
        assert!(height(PASS.to) - height(MORIA.from) > 30.0);
        // Down to the Anduin's water, from the mines' gate.
        assert!(height(MORIA.to) - height(CAUSEWAY.from + 50.0) > 20.0);
    }

    #[test]
    fn the_road_winds_in_esses_through_the_trollshaws_and_the_gorge() {
        crate::game::register();
        // Bends one after another each the other way, tighter than 70 m.
        let track = Track::new(&fellowship_road().layout);
        for stretch in [TROLLSHAWS, GORGE] {
            let bends: Vec<f32> = (track.index_at(stretch.from)..track.index_at(stretch.to))
                .map(|index| track.curvature(index))
                .filter(|curvature| curvature.abs() > 1.0 / 70.0)
                .fold(Vec::new(), |mut bends, curvature| {
                    if bends
                        .last()
                        .is_none_or(|last: &f32| last.signum() != curvature.signum())
                    {
                        bends.push(curvature);
                    }
                    bends
                });
            assert!(
                bends.len() >= 3,
                "{} bends from {}",
                bends.len(),
                stretch.from
            );
        }
    }

    #[test]
    fn the_road_runs_down_the_anduin_between_the_kings_and_is_dry_but_on_its_bridges() {
        crate::game::register();
        // The water as the circuit's file has it.
        let Coast::Inland(water) = fellowship_road().scenery.coast else {
            panic!("Fellowship Road's coast is its water");
        };
        let track = Track::new(&fellowship_road().layout);
        let lap = track.lap_length();
        let on_bridge = |along: f32| {
            fellowship_road()
                .scenery
                .bridges
                .iter()
                .any(|bridge| bridge.span.contains(along, lap))
        };
        // Wet only on the bridges; each of them over water.
        let mut wet = vec![0.0; fellowship_road().scenery.bridges.len()];
        for index in 0..track.sample_count() {
            let along = index as f32 * track.step_length();
            if water.distance(track.centerline()[index]) < 0.0 {
                assert!(on_bridge(along), "wet road at {along} m");
                for (k, bridge) in fellowship_road().scenery.bridges.iter().enumerate() {
                    if bridge.span.contains(along, lap) {
                        wet[k] += track.step_length();
                    }
                }
            }
        }
        for (k, bridge) in fellowship_road().scenery.bridges.iter().enumerate() {
            assert!(wet[k] > 0.5 * bridge.span.length(lap), "{wet:?}");
        }
        // Down the middle of the river all along the causeway, the water
        // either side as far as the kings and past them.
        assert!(
            wet[2] > 0.85 * CAUSEWAY.length(lap),
            "{} m over water",
            wet[2]
        );
        for king in KINGS {
            let index = track.nearest_index(king);
            let along = index as f32 * track.step_length();
            assert!(CAUSEWAY.contains(along, lap));
            let road = track.centerline()[index];
            let beyond = road + (king - road) * 1.25;
            assert!(water.distance(beyond) < 0.0, "{beyond}");
            assert!((20.0..26.0).contains(&king.distance(road)));
        }
    }

    #[test]
    fn the_misty_mountains_rise_under_snow_to_the_east_and_nothing_by_the_road() {
        crate::game::register();
        let track = Track::new(&fellowship_road().layout);
        for &point in track.centerline() {
            assert!(mountains(point) < 1e-3, "{point}");
        }
        // Peaks over the snow line (560 m) along the range to the east.
        let highest = (0..60)
            .map(|k| mountains(Vec2::new(3000.0, -1500.0 + k as f32 * 50.0)))
            .fold(0.0, f32::max);
        assert!(highest > 650.0, "{highest} m");
        let west = (0..60)
            .map(|k| mountains(Vec2::new(-1800.0, -1500.0 + k as f32 * 50.0)))
            .fold(0.0, f32::max);
        assert!(west < 200.0, "{west} m");
    }
}
