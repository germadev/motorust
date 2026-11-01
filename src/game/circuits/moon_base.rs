//! Base Lunar (Moon Base): a lap that starts at an Earth Space Launch &
//! Deep-Space Observatory Complex in the coastal mountains, crosses an
//! interdimensional portal to a lunar colony in Mare Tranquillitatis, and
//! returns through a second portal to Earth's high peaks.
//!
//! 1. **Earth Space Center & Observatory Peak (`0..1120 m`):** The start
//!    straight runs north on Earth past a Saturn-V space launchpad, an
//!    astronomical telescope observatory dome (`Observatory`), and giant
//!    deep-space radio antennas (`RadioDish`), with the Moon shining in the
//!    night sky overhead. After a straightaway jump crest (`55 m`), the road
//!    climbs +31 m up the forested Observatory Mountain (`32 m`), plunges
//!    -31.6 m into the underground Launch Complex Tunnel (`800..1020 m`), and
//!    reaches the first **Interdimensional Portal (`Earth → Moon`, `1120 m`)**.
//! 2. **Lunar Colony & Crater Split (`1120..2895 m`):** Emerging onto the
//!    airless grey regolith of the Moon (where the blue Earth hangs in the
//!    sky), riders cross a steel truss bridge over a lunar rille, pass
//!    pressurized geodesic biodomes, six-wheeled lunar rovers, and solar
//!    arrays, sweep round the Sea of Tranquillity crater, and climb a two-way
//!    Crater Split (`2340..2690 m`) to the second **Interdimensional Portal
//!    (`Moon → Earth`, `2895 m`)**.
//! 3. **High Earth Summit & Western Mountain Roller-Coaster (`2895..4600 m`):**
//!    Back on Earth at `42.5..45.5 m` altitude, the road crosses 45 m over the
//!    top of the tunnel, plunges -40 m into a deep forested valley (`5.5 m`),
//!    climbs +30 m back up the Western Coastal Range (`35.6 m`), launches off a
//!    high-altitude straightaway jump (`4095 m`), and dives -35.6 m back to the
//!    finish line.
//!
//! The circuit is written in RON, `assets/circuits/moon_base.ron`, its
//! mountains too; here are tests of it.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::circuits::Circuit;
    use crate::game::CIRCUITS;
    use crate::track::Track;
    use crate::world::is_earth_realm;
    use crate::world::scenery::{Span, span};

    /// Where things are, as in the circuit's file (m from the finish line).
    const TUNNEL: Span = span(800.0, 1020.0);
    const CROSSING: Span = span(2970.0, 3140.0);
    const MOON_PORTAL_IN: f32 = 1120.0;
    const EARTH_PORTAL_OUT: f32 = 2895.0;
    /// The massif the tunnel runs under and the road back climbs over.
    const STATION: Vec2 = Vec2::new(550.0, -430.0);
    const STATION_RADIUS: f32 = 220.0;

    fn moon_base() -> &'static Circuit {
        CIRCUITS.iter().find(|c| c.name_en == "Moon Base").unwrap()
    }

    #[test]
    fn the_lap_goes_through_the_space_station_tunnel_and_then_over_it() {
        let track = Track::new(&moon_base().layout);
        let line = track.centerline();
        let middle = line[track.index_at((TUNNEL.from + TUNNEL.to) / 2.0)];
        assert!(middle.distance(STATION) < 0.2 * STATION_RADIUS);
        assert!(
            moon_base()
                .scenery
                .massifs
                .iter()
                .any(|m| m.center == STATION && m.radius == STATION_RADIUS)
        );

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
        assert!(above > 40.0, "{above} m above");
    }

    #[test]
    fn the_lap_starts_on_earth_with_launchpads_and_observatories_and_crosses_portals_to_the_moon() {
        let scenery = &moon_base().scenery;
        let space = scenery
            .themes
            .get::<crate::game::space::Space>()
            .expect("Moon Base has Space");
        assert_eq!(scenery.portals.len(), 2);
        assert!(!space.observatories.is_empty());
        assert!(!space.rockets.is_empty());
        assert!(!space.dishes.is_empty());

        // All launchpads (rockets), observatories, and giant radio dishes stand on Earth.
        for rocket in space.rockets {
            assert!(is_earth_realm(rocket.position), "rocket on Earth");
        }
        for obs in space.observatories {
            assert!(is_earth_realm(obs.position), "observatory on Earth");
        }
        for dish in space.dishes {
            assert!(is_earth_realm(dish.position), "radio dish on Earth");
        }

        let track = Track::new(&moon_base().layout);
        let line = track.centerline();
        let lap = track.lap_length();

        // Every observatory, rocket launchpad, and giant radio dish stands right
        // beside the road (`22..38 m` from the centerline) so it towers in the
        // rider's view without encroaching on the asphalt.
        for obs in space.observatories {
            let d = track.distance_to_centerline(obs.position);
            assert!(
                (22.0..=38.0).contains(&d),
                "observatory at {:?} is {d:.1} m from road centerline",
                obs.position
            );
        }
        for rocket in space.rockets {
            let d = track.distance_to_centerline(rocket.position);
            assert!(
                (22.0..=38.0).contains(&d),
                "rocket at {:?} is {d:.1} m from road centerline",
                rocket.position
            );
        }
        for dish in space.dishes {
            let d = track.distance_to_centerline(dish.position);
            assert!(
                (20.0..=38.0).contains(&d),
                "radio dish at {:?} is {d:.1} m from road centerline",
                dish.position
            );
        }
        assert!(space.arches.len() >= 12);
        assert!(space.biodomes.len() >= 8);

        // The circuit starts on Earth (along = 0 m), and the Earth stretch is longer than the Moon stretch.
        assert!(is_earth_realm(line[0]), "starts on Earth");
        let moon_length = EARTH_PORTAL_OUT - MOON_PORTAL_IN;
        let earth_length = lap - moon_length;
        assert!(
            earth_length > moon_length + 800.0,
            "Earth stretch ({earth_length:.0} m) is longer than Moon stretch ({moon_length:.0} m)"
        );

        // Before the first portal (on Earth) vs after it (on the Moon).
        assert!(is_earth_realm(line[track.index_at(MOON_PORTAL_IN - 25.0)]));
        assert!(!is_earth_realm(line[track.index_at(MOON_PORTAL_IN + 25.0)]));
        // Before the second portal (on the Moon) vs after it (back on Earth).
        assert!(!is_earth_realm(
            line[track.index_at(EARTH_PORTAL_OUT - 25.0)]
        ));
        assert!(is_earth_realm(
            line[track.index_at(EARTH_PORTAL_OUT + 25.0)]
        ));
    }
}
