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
//! The circuit is written in RON, `assets/circuits/fellowship_road.ron`;
//! here are only the functions its file names, its water and its
//! mountains, and tests of it.

use bevy::prelude::*;

use crate::geometry::fbm;

/// The pool of Bywater, below the right of the village, where the Water
/// widens before it runs on into the Brandywine: its middle and radius
/// (m).
const POND: Vec2 = Vec2::new(470.0, -330.0);
const POND_RADIUS: f32 = 30.0;
/// The Water, the stream through Hobbiton, 64 m wide: out of the wood,
/// along the left of the long sweep down into the village, between the
/// road and the mill; square under the bridge, along the right of the
/// sweep on, and away from the road into the pool, and on out of it into
/// the Brandywine.
const THE_WATER: [(Vec2, f32); 23] = [
    (Vec2::new(-150.0, 40.0), 8.0),
    (Vec2::new(-115.0, -60.0), 9.0),
    (Vec2::new(-75.0, -110.0), 12.0),
    (Vec2::new(-59.7, -132.0), 16.0),
    (Vec2::new(-53.6, -179.1), 22.0),
    (Vec2::new(-39.3, -222.1), 28.0),
    (Vec2::new(-12.6, -268.1), 32.0),
    (Vec2::new(23.7, -307.0), 32.0),
    (Vec2::new(67.6, -337.1), 32.0),
    (Vec2::new(111.6, -355.1), 32.0),
    (Vec2::new(138.2, -362.1), 32.0),
    (Vec2::new(154.0, -364.6), 32.0),
    (Vec2::new(168.6, -346.7), 32.0),
    (Vec2::new(187.2, -312.2), 32.0),
    (Vec2::new(205.8, -277.8), 32.0),
    (Vec2::new(220.5, -259.8), 32.0),
    (Vec2::new(234.5, -261.9), 32.0),
    (Vec2::new(256.1, -267.5), 32.0),
    (Vec2::new(289.7, -280.1), 32.0),
    (Vec2::new(440.0, -345.0), 32.0),
    (Vec2::new(470.0, -330.0), 30.0),
    (Vec2::new(505.0, -332.0), 16.0),
    (Vec2::new(548.0, -330.0), 18.0),
];
/// Rivers, as points (x, z) along them with how far their banks are from
/// the middle there (m): the Brandywine, north to south under its bridge
/// into a pool; the Bruinen, from the mountains west across the ford into
/// a pool; and the Anduin, from the south into the middle of the road,
/// widening down the causeway to the kings, narrowing past them through the
/// gorge in the rock they guard, and turning into the lake.
const BRANDYWINE_RIVER: [(Vec2, f32); 5] = [
    (Vec2::new(600.0, -1100.0), 18.0),
    (Vec2::new(570.0, -800.0), 18.0),
    (Vec2::new(560.0, -568.0), 18.0),
    (Vec2::new(550.0, -330.0), 18.0),
    (Vec2::new(520.0, -200.0), 40.0),
];
const BRUINEN_RIVER: [(Vec2, f32); 6] = [
    (Vec2::new(1985.0, -565.0), 22.0),
    (Vec2::new(1735.0, -455.0), 22.0),
    (Vec2::new(1576.0, -417.0), 22.0),
    (Vec2::new(1435.0, -375.0), 22.0),
    (Vec2::new(1325.0, -335.0), 22.0),
    (Vec2::new(1235.0, -305.0), 40.0),
];
const ANDUIN: [(Vec2, f32); 13] = [
    (Vec2::new(680.0, 800.0), 18.0),
    (Vec2::new(630.0, 600.0), 18.0),
    (Vec2::new(598.0, 470.0), 14.0),
    (Vec2::new(578.0, 398.0), 12.0),
    (Vec2::new(545.0, 396.5), 45.0),
    (Vec2::new(495.0, 389.5), 55.0),
    // Between the kings, where the cliffs close in on it, and past them
    // through the gorge in the rock.
    (Vec2::new(434.0, 372.5), 35.0),
    (Vec2::new(405.0, 362.0), 31.0),
    (Vec2::new(351.0, 350.0), 29.0),
    (Vec2::new(312.0, 360.0), 28.0),
    (Vec2::new(282.0, 405.0), 30.0),
    (Vec2::new(228.0, 452.0), 30.0),
    (Vec2::new(205.0, 478.0), 30.0),
];
/// Mirrormere, below Moria's gate, and Nen Hithoel, the lake the Anduin
/// opens into past the kings: their middles and radii (m).
const MIRRORMERE: Vec2 = Vec2::new(1350.0, 505.0);
const MIRRORMERE_RADIUS: f32 = 45.0;
const NEN_HITHOEL: Vec2 = Vec2::new(180.0, 500.0);
const NEN_HITHOEL_RADIUS: f32 = 85.0;

/// How far `point` is from the shores of the rivers and lakes: negative in
/// their water. Their shores wander a little.
pub(super) fn water(point: Vec2) -> f32 {
    let wander = 4.0 * (2.0 * fbm(point / 40.0 + 7.0, 2) - 1.0);
    let pond = point.distance(POND) - POND_RADIUS;
    let mirrormere = point.distance(MIRRORMERE) - MIRRORMERE_RADIUS;
    let lake = point.distance(NEN_HITHOEL) - NEN_HITHOEL_RADIUS;
    let rivers = [&THE_WATER[..], &BRANDYWINE_RIVER, &BRUINEN_RIVER, &ANDUIN]
        .into_iter()
        .map(|river| river_banks(river, point))
        .fold(f32::MAX, f32::min);
    pond.min(mirrormere).min(lake).min(rivers) + wander
}

/// How far `point` is from the banks of a river through `points` (each
/// with how far its banks are from its middle there): negative in it.
fn river_banks(points: &[(Vec2, f32)], point: Vec2) -> f32 {
    points
        .windows(2)
        .map(|pair| {
            let ((a, wide_a), (b, wide_b)) = (pair[0], pair[1]);
            let ab = b - a;
            let t = ((point - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            point.distance(a + ab * t) - wide_a.lerp(wide_b, t)
        })
        .fold(f32::MAX, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How high the circuit's mountains stand at `point`, as its file has
    /// them.
    fn mountains(point: Vec2) -> f32 {
        fellowship_road().scenery.mountains.height(point)
    }
    use crate::circuits::{CIRCUITS, Circuit};
    use crate::track::Track;
    use crate::world::scenery::{Coast, Span, span};

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
            if water(track.centerline()[index]) < 0.0 {
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
            assert!(water(beyond) < 0.0, "{beyond}");
            assert!((20.0..26.0).contains(&king.distance(road)));
        }
    }

    #[test]
    fn the_misty_mountains_rise_under_snow_to_the_east_and_nothing_by_the_road() {
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
