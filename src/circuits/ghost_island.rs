//! Ghost Island: a lap of an island of feudal Japan, after Ghost of
//! Tsushima, its stretches each in a colour of their own. It crosses the
//! castle hill twice, in a figure of eight: first through it, by a tunnel at
//! the end of the castle street, and at the end of the lap over it, 33 m
//! above, through the castle.
//!
//! 1. The castle street, north, over a crest where the bike takes off,
//!    straight at the castle hill and into the tunnel under it, at its foot.
//! 2. Out onto the coast, round to the right, and east along it in esses,
//!    past the great torii standing in the sea.
//! 3. The shrine on the water: a boardwalk, narrower than the road, straight
//!    across an inlet, with a crest in the middle where the bike takes off.
//! 4. Up the susuki hills in esses at up to 12 %, south over their rolling
//!    top, two crests where the bike takes off, and down at 12 % to the lake.
//! 5. Round the lake, clockwise, three sides of it, along a shrine's
//!    vermilion fence on the lake's side: fields of flowers down to the
//!    water, a water wheel on its shore and, on the outside of the bend at
//!    its foot, the great maple over its clearing of red leaves.
//! 6. North from the lake to the temple, where the road splits in two: the
//!    lap snakes through its courtyard and a way to the left swings out
//!    early over a hump.
//! 7. Up the red ridge in bends at up to 12 %, most of the way through a
//!    narrow corridor of torii, as at Fushimi Inari in Kyoto, to its top,
//!    53 m up, and round to the left onto the castle ridge, west.
//! 8. Down along the castle ridge, where the road splits three ways again,
//!    the other way round, and over the tunnel through the castle hill.
//! 9. Up the bamboo hill at 12 %, and down it through the bamboo at 11 % in
//!    a run of tight esses, four bends winding like a river, and round to
//!    the left into the castle street.
//!
//! The circuit is written in RON, `assets/circuits/ghost_island.ron`; here
//! are only the functions its file names, its lake and its mountains (the
//! volcano), and tests of it.

use bevy::prelude::*;

use crate::geometry::{fbm, smoothstep};

/// The lake the road goes round: its middle, the radius of its shore (m)
/// and how far the shore wanders (m); and the islet in it, with a wayside
/// shrine.
const LAKE: Vec2 = Vec2::new(870.0, 270.0);
const LAKE_RADIUS: f32 = 150.0;
const LAKE_WANDER: f32 = 8.0;
const ISLET: Vec2 = Vec2::new(885.0, 255.0);
const ISLET_RADIUS: f32 = 16.0;

/// How far `point` is from the lake's shore: negative in its water, and
/// positive on the islet.
pub(super) fn lakes(point: Vec2) -> f32 {
    let wander = LAKE_WANDER * (2.0 * fbm(point / 50.0 + 11.0, 2) - 1.0);
    let lake = point.distance(LAKE) - LAKE_RADIUS + wander;
    let islet = ISLET_RADIUS - point.distance(ISLET);
    lake.max(islet)
}

/// The volcano across the strait to the north, as Mount Yōtei over Hokkaido
/// or Fuji over its lakes: its middle, its radius and its height (m). Its
/// foot rises out of the sea just behind the great torii, and from the
/// castle street it stands over the castle; its top is under snow (see
/// `SNOW_LINE` in `crate::world::terrain`), the sky clear enough to see it
/// sharp from across the island.
const VOLCANO: Vec2 = Vec2::new(430.0, -2750.0);
const VOLCANO_RADIUS: f32 = 1650.0;
const VOLCANO_HEIGHT: f32 = 900.0;

/// Across the sea: the volcano to the north, a lone cone with its flanks
/// sweeping up ever steeper to a crater at its top, and lower mountains
/// round the rest of the horizon, clear of it.
pub(super) fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(450.0, -150.0);
    let reach = point.distance(VOLCANO) / VOLCANO_RADIUS;
    let ranges = smoothstep(1900.0, 2800.0, point.distance(CENTER))
        * smoothstep(1.1, 1.7, reach)
        * 220.0
        * (0.35 + 0.65 * fbm(point / 300.0, 4));
    let cone = VOLCANO_HEIGHT * (1.0 - reach).max(0.0).powf(1.7);
    let crater = 40.0 * (1.0 - smoothstep(0.0, 0.08, reach));
    ranges.max(cone - crater + volcano_relief(point, reach))
}

/// The volcano's flanks as erosion and its own fires have left them (m over
/// or under its smooth cone, `reach` its share of the way out from its
/// middle): gullies carved down them, rounded at their bottoms, between
/// ridges that are the cone itself (so its skyline stays smooth), deepest
/// half way down and none at the rim nor at the foot, each wandering a
/// little as it runs down; broad lumps and hollows; and a parasitic cone on
/// the flank facing the island.
fn volcano_relief(point: Vec2, reach: f32) -> f32 {
    if reach >= 1.0 {
        return 0.0;
    }
    let angle = (point - VOLCANO).to_angle();
    let mut gullies = 0.0;
    // Each set of gullies only where they are far enough apart for the
    // land's grid (about 70 m there) to draw them.
    for (k, (count, weight, from)) in [(13.0, 1.0, 0.18), (21.0, 0.6, 0.28), (34.0, 0.35, 0.46)]
        .into_iter()
        .enumerate()
    {
        let wander = 2.5 * (fbm(Vec2::new(reach * 5.0, k as f32 * 3.7), 2) - 0.5);
        let wave = (count * angle + wander + k as f32 * 1.3).sin();
        gullies += weight * wave.max(0.0).powi(2) * smoothstep(from, from + 0.1, reach);
    }
    let down = 1.0 - smoothstep(0.75, 1.0, reach);
    let lumps = 60.0 * (fbm(point / 420.0, 3) - 0.5) * smoothstep(0.1, 0.3, reach) * down;
    let parasite = VOLCANO + Vec2::new(0.5, 0.86) * 0.55 * VOLCANO_RADIUS;
    let near = point.distance(parasite) / 230.0;
    let cone =
        95.0 * (1.0 - near * near).max(0.0).powi(2) - 18.0 * (1.0 - smoothstep(0.0, 0.18, near));
    lumps + cone - 45.0 * gullies * down
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::{CIRCUITS, Circuit};
    use crate::track::Track;
    use crate::world::scenery::{Coast, Span, span};

    /// Where things are, as in the circuit's file (m from the finish line).
    const TUNNEL: Span = span(360.0, 620.0);
    const BOARDWALK: Span = span(1422.0, 1622.0);
    /// From the end of the boardwalk over the susuki hills to the lake.
    const SUSUKI_HILLS: Span = span(1622.0, 2700.0);
    const LAKE_SHORE: Span = span(2700.0, 3400.0);
    const RED_RIDGE: Span = span(3764.0, 4322.0);
    const CROSSING: Span = span(4672.0, 4880.0);
    const BAMBOO: Span = span(4880.0, 5675.0);
    /// The castle hill the castle street runs into, by a tunnel, and the
    /// lap comes back over: its middle and its radius (m).
    const CASTLE_HILL: Vec2 = Vec2::new(0.0, -530.0);
    const CASTLE_HILL_RADIUS: f32 = 210.0;

    fn ghost_island() -> &'static Circuit {
        CIRCUITS
            .iter()
            .find(|c| c.name_en == "Ghost Island")
            .unwrap()
    }

    #[test]
    fn the_lap_goes_through_the_castle_hill_and_then_over_it() {
        let track = Track::new(&ghost_island().layout);
        let line = track.centerline();
        // Through it, by its middle.
        let middle = line[track.index_at((TUNNEL.from + TUNNEL.to) / 2.0)];
        assert!(middle.distance(CASTLE_HILL) < 0.2 * CASTLE_HILL_RADIUS);
        assert!(
            ghost_island()
                .scenery
                .massifs
                .iter()
                .any(|m| m.center == CASTLE_HILL && m.radius == CASTLE_HILL_RADIUS)
        );
        // Over it at the end of the lap, well above the tunnel.
        let under = track.index_at(TUNNEL.from)..track.index_at(TUNNEL.to);
        let (gap, over, under) = (track.index_at(CROSSING.from)..track.index_at(CROSSING.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the crossing and the tunnel have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 25.0, "{above} m above");
    }

    #[test]
    fn the_hills_are_steep_and_the_road_climbs_and_drops_a_lot() {
        let track = Track::new(&ghost_island().layout);
        let step = track.step_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let grade = |along: f32| {
            let index = track.index_at(along);
            (track.elevations()[index + 1] - track.elevations()[index]) / step
        };
        let steepest = |stretch: Span| {
            (track.index_at(stretch.from)..track.index_at(stretch.to))
                .map(|index| grade(index as f32 * step).abs())
                .fold(0.0, f32::max)
        };
        // Up the susuki hills, and down them to the lake, at nearly 12 %.
        assert!(height(2300.0) - height(SUSUKI_HILLS.from) > 30.0);
        assert!(steepest(span(1700.0, 2300.0)) > 0.115);
        assert!(steepest(span(2600.0, 2900.0)) > 0.115);
        // Up the red ridge to its top, 30 m and more, and down from the
        // castle to town.
        let top = (track.index_at(RED_RIDGE.from)..track.index_at(RED_RIDGE.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(top - height(RED_RIDGE.from) > 30.0, "{top} m at the top");
        assert!(steepest(RED_RIDGE) > 0.115);
        assert!(height(CROSSING.from) - height(0.0) > 35.0);
        // Up the bamboo hill, and down it, 40 m and more, at 11 % most of
        // the way.
        let hill = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| track.elevations()[index])
            .fold(f32::MIN, f32::max);
        assert!(hill - height(CROSSING.to) > 12.0, "{hill} m at the top");
        assert!(hill - height(BAMBOO.to) > 40.0, "{hill} m at the top");
        assert!(steepest(span(CROSSING.to, CROSSING.to + 150.0)) > 0.115);
        let down = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .filter(|&index| grade(index as f32 * step) < -0.105)
            .count() as f32
            * step;
        assert!(down > 200.0, "{down} m at 11 %");
    }

    #[test]
    fn the_road_winds_down_the_bamboo_hill_in_tight_esses() {
        // Four tight bends and more, one after the other, each the other
        // way.
        let track = Track::new(&ghost_island().layout);
        let bends: Vec<f32> = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| track.curvature(index))
            .filter(|curvature| curvature.abs() > 1.0 / 50.0)
            .fold(Vec::new(), |mut bends, curvature| {
                if bends
                    .last()
                    .is_none_or(|last: &f32| last.signum() != curvature.signum())
                {
                    bends.push(curvature);
                }
                bends
            });
        assert!(bends.len() >= 5, "{} bends", bends.len());
        let tightest = (track.index_at(BAMBOO.from)..track.index_at(BAMBOO.to))
            .map(|index| 1.0 / track.curvature(index).abs())
            .fold(f32::MAX, f32::min);
        assert!(tightest < 42.0, "{tightest} m");
    }

    #[test]
    fn the_road_rounds_a_lake_with_an_islet_and_crosses_an_inlet_on_the_boardwalk() {
        // The island's shore and its lake as the circuit's file has them.
        let Coast::OutlineWithLakes(island, lakes) = ghost_island().scenery.coast else {
            panic!("Ghost Island is an island with a lake");
        };
        let track = Track::new(&ghost_island().layout);
        let lap = track.lap_length();
        // Three sides of the lake, close by its shore all the way, with the
        // lake on the right, and low, just over the water.
        let mut round = 0.0;
        for index in 0..track.sample_count() {
            let point = track.centerline()[index];
            let shore = lakes(point);
            assert!(shore > 15.0, "{shore} m from the lake at {index}");
            let along = index as f32 * track.step_length();
            if span(2950.0, 3250.0).contains(along, lap) {
                let height = track.elevations()[index];
                assert!(height < 6.0, "{height} m up by the lake at {along}");
            }
            if LAKE_SHORE.contains(along, lap) && shore < 40.0 {
                round += track.step_length();
                let lake_side = (LAKE - point).dot(track.offset_point(index, 1.0) - point);
                assert!(lake_side < 0.0, "the lake on the left at {index}");
            }
        }
        assert!(round > 450.0, "{round} m round the lake");
        assert!(lakes(LAKE + Vec2::new(-40.0, 30.0)) < -20.0, "deep water");
        assert!(lakes(ISLET) > 10.0, "an islet");
        // The boardwalk crosses the inlet; the water reaches in under it.
        let middle = track.centerline()[track.index_at((BOARDWALK.from + BOARDWALK.to) / 2.0)];
        let shore = crate::track::closed_curve(island, 4.0);
        let inside = |point: Vec2| {
            let n = shore.len();
            (0..n).fold(false, |inside, i| {
                let (a, b) = (shore[i], shore[(i + 1) % n]);
                let crosses = (a.y > point.y) != (b.y > point.y)
                    && point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
                inside != crosses
            })
        };
        assert!(!inside(middle), "the inlet is under the boardwalk");
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.from - 30.0)]
        ));
        assert!(inside(
            track.centerline()[track.index_at(BOARDWALK.to + 30.0)]
        ));
    }

    #[test]
    fn the_volcanos_flanks_are_carved_into_gullies_and_ridges() {
        // Half way down, round it: ridges and gullies one after another, tens
        // of metres apart in height, not a smooth cone.
        let heights: Vec<f32> = (0..720)
            .map(|k| {
                let away = Vec2::from_angle(k as f32 / 720.0 * std::f32::consts::TAU);
                mountains(VOLCANO + away * 0.5 * VOLCANO_RADIUS)
            })
            .collect();
        let (low, high) = heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &y| (l.min(y), h.max(y)));
        assert!(high - low > 50.0, "{} m from gully to ridge", high - low);
        let turns = heights
            .windows(3)
            .filter(|w| (w[1] - w[0]) * (w[2] - w[1]) < 0.0)
            .count();
        assert!(turns > 40, "{turns} ridges and gullies round it");
        // None at the rim, nor past the foot.
        let rim = mountains(VOLCANO + Vec2::X * 0.1 * VOLCANO_RADIUS);
        let smooth = VOLCANO_HEIGHT * 0.9f32.powf(1.7) - 40.0 * (1.0 - smoothstep(0.0, 0.08, 0.1));
        assert!((rim - smooth).abs() < 1.0, "{rim} at the rim");
    }

    #[test]
    fn the_volcano_rises_across_the_strait_and_nothing_is_raised_by_the_road() {
        let track = Track::new(&ghost_island().layout);
        assert!(mountains(VOLCANO) > 0.85 * VOLCANO_HEIGHT);
        assert!(mountains(VOLCANO) < VOLCANO_HEIGHT, "a crater at its top");
        // Under snow at its top, with its rim, and alone: no mountain
        // beside it reaches a third of its height.
        let rim = VOLCANO + Vec2::X * 0.1 * VOLCANO_RADIUS;
        assert!(mountains(rim) > 600.0, "{}", mountains(rim));
        for k in 0..36 {
            let away = Vec2::from_angle(k as f32 / 36.0 * std::f32::consts::TAU);
            let beside = VOLCANO + away * 1.3 * VOLCANO_RADIUS;
            assert!(mountains(beside) < VOLCANO_HEIGHT / 3.0, "{beside}");
        }
        // Straight ahead, over the castle, down the castle street.
        let street = track.centerline()[track.index_at(100.0)];
        let ahead = track.centerline()[track.index_at(300.0)] - street;
        let toward = VOLCANO - street;
        assert!(
            ahead.angle_to(toward).abs() < 0.3,
            "{}",
            ahead.angle_to(toward)
        );
        for &point in track.centerline() {
            assert!(mountains(point) < 1e-3, "{point}");
        }
    }
}
