//! Gold Mine: a lap through a gold mine in a desert at dusk, after Wario's
//! Gold Mine (Mario Kart) and Dragon Mines (Crash Team Racing), with the
//! roads of West Way (Moto Racer) round it: ore carts on rails, timber
//! trestles and headframes, red rock, dunes and a low orange sun. It
//! crosses the mountain twice, in a figure of eight: first through it, by
//! the galleries of the mine, and then over it.
//!
//! 1. The start straight runs north through the mining town, past its
//!    wooden houses, lanterns and a train of ore carts, over a crest where
//!    the bike takes off.
//! 2. Into the dunes: up and down over them, past a gas station on a blind
//!    crest where the bike takes off, and round their far end.
//! 3. South down a narrow red canyon in esses, between sheer walls, to the
//!    east mouth of the excavation, under a trestle where a cart crosses.
//! 4. Through the mountain (900 m), east to west: a timbered gallery, halls
//!    of sandstone, slate and clay, down into the mine and up again, and
//!    twice the road splits three ways, far apart, each way with its own
//!    climb or drop: a high road and a deep one, an upper gallery and a
//!    lower one, and the lap snaking between them. None is clearly the
//!    quickest. Between the splits, a timbered neck runs under the road over
//!    the top.
//! 5. Out of the west mouth and along the rim of a great open pit.
//! 6. The wooden bridge, straight across the lake at the bottom of the pit
//!    for nearly half a kilometre, 18 m over the red water, with a crest in
//!    the middle where the bike takes off.
//! 7. Up the switchbacks of the mountain's north flank, round two hairpins,
//!    and a long sweep onto the top, 55 m up.
//! 8. Over the top, past a headframe, across the road through the mountain
//!    70 m beneath.
//! 9. Down its south-east flank in esses and back into town.
//!
//! The circuit is written in RON, `assets/circuits/gold_mine.ron`, its
//! water and its mountains too; here are tests of it.

mod tests {
    use bevy::prelude::*;

    use crate::circuits::Circuit;
    use crate::game::CIRCUITS;
    use crate::track::Track;
    use crate::world::scenery::{Coast, Span, span};

    /// Where things are, as in the circuit's file (m from the finish line).
    const EXCAVATION: Span = span(1655.0, 2555.0);
    const BRIDGE: Span = span(2905.0, 3425.0);
    const CLIMB: Span = span(3425.0, 4580.0);
    const SUMMIT: Span = span(4580.0, 5000.0);
    const DESCENT: Span = span(5000.0, 5780.0);
    /// The great lake at the bottom of the pit: its middle (m).
    const LAKE: Vec2 = Vec2::new(-1225.0, -626.0);
    /// The mountain the lap crosses through and over: its middle and its
    /// radius (m).
    const MOUNTAIN: Vec2 = Vec2::new(-720.0, -200.0);
    const MOUNTAIN_RADIUS: f32 = 540.0;

    fn gold_mine() -> &'static Circuit {
        CIRCUITS.iter().find(|c| c.name_en == "Gold Mine").unwrap()
    }

    #[test]
    fn the_road_crosses_the_great_lake_on_the_long_bridge() {
        // The water as the circuit's file has it.
        let Coast::Sea(water) = gold_mine().scenery.coast else {
            panic!("Gold Mine's coast is its water");
        };
        let track = Track::new(&gold_mine().layout);
        let lap = track.lap_length();
        // The road crosses the water only on the bridge, and the bridge
        // nearly half a kilometre long, over water most of the way.
        let mut wet = 0.0;
        for index in 0..track.sample_count() {
            let along = index as f32 * track.step_length();
            let over_water = water.distance(track.centerline()[index]) < 0.0;
            assert!(
                !over_water || BRIDGE.contains(along, lap),
                "wet road at {along} m"
            );
            if over_water {
                wet += track.step_length();
            }
        }
        assert!(BRIDGE.length(lap) > 450.0);
        assert!(wet > 0.85 * BRIDGE.length(lap), "{wet} m over water");
        // A great lake, and the town dry.
        assert!(water.distance(LAKE) < -100.0);
        assert!(water.distance(LAKE + Vec2::new(200.0, 0.0)) < 0.0);
        let town = track.centerline()[track.index_at(150.0)];
        assert!(water.distance(town) > 100.0, "town is dry");
    }

    #[test]
    fn the_lap_crosses_the_mountain_through_it_and_then_over_it() {
        let track = Track::new(&gold_mine().layout);
        let step = track.step_length();
        let lap = track.lap_length();
        let height = |along: f32| track.elevations()[track.index_at(along)];
        let point = |along: f32| track.centerline()[track.index_at(along)];
        // Through it, from one side to the other, by its middle.
        for mouth in [EXCAVATION.from, EXCAVATION.to] {
            let out = point(mouth).distance(MOUNTAIN) / MOUNTAIN_RADIUS;
            assert!(out > 0.7, "a mouth {out} of the way out");
        }
        let middle = point(EXCAVATION.from + EXCAVATION.length(lap) / 2.0);
        let deep = middle.distance(MOUNTAIN);
        assert!(deep < 0.2 * MOUNTAIN_RADIUS, "{deep} m from its middle");
        assert!(
            gold_mine()
                .scenery
                .massifs
                .iter()
                .any(|m| m.center == MOUNTAIN && m.radius == MOUNTAIN_RADIUS)
        );
        // Then up it, 45 m and more, steadily, over the top, and down it,
        // 60 m and more, back into town.
        assert!(height(CLIMB.to) - height(CLIMB.from) > 45.0);
        assert!(height(DESCENT.from) - height(0.0) > 60.0);
        let (first, last) = (track.index_at(CLIMB.from + 100.0), track.index_at(CLIMB.to));
        for index in first..last {
            let grade = (track.elevations()[index + 1] - track.elevations()[index]) / step;
            assert!(grade > 0.03, "{grade} at {} m", index as f32 * step);
        }
        // Over the top, the road crosses the road through the mountain,
        // high above it.
        let line = track.centerline();
        let under = track.index_at(EXCAVATION.from)..track.index_at(EXCAVATION.to);
        let (gap, over, under) = (track.index_at(SUMMIT.from)..track.index_at(SUMMIT.to))
            .flat_map(|over| {
                under
                    .clone()
                    .map(move |under| (line[over].distance(line[under]), over, under))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the summit and the excavation have points");
        assert!(gap < 3.0, "{gap} m apart at the closest");
        let above = track.elevations()[over] - track.elevations()[under];
        assert!(above > 50.0, "{above} m above");
    }
}
