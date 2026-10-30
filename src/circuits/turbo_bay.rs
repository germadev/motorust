//! Turbo Bay, inspired by Speed Bay in Moto Racer: a figure of eight on a
//! hilly island, with the sea all around and two bays cutting in towards
//! its waist. The first loop goes clockwise and the second anticlockwise,
//! and halfway round the road climbs back over the ridge it first went
//! through in a tunnel.
//!
//! 1. The start straight is a seafront promenade heading north: railing,
//!    beach and sea on the left; palms and tall, colourful hotels on the
//!    right.
//! 2. On level into a steep banked right-hander at the water's edge, and
//!    the road climbs east into a tunnel through the ridge at the island's
//!    waist.
//! 3. Out of the tunnel, past an old-style grand hotel on a headland above
//!    the south-east bay, to a pass between rocks.
//! 4. The east coast road rises and falls behind a white barrier, with the
//!    sea on the right.
//! 5. A mountain road climbs on north in S-bends between rock walls, 37 m
//!    up, over a broad top with the lighthouse straight ahead below, and
//!    dives steeply at it to the foot of its tower on the northern cape,
//!    round a long left-hander.
//! 6. Back up through a forest it climbs, eases off over a broad shoulder and
//!    rises again, then
//!    runs down the ridge and crosses over the tunnel, down the shore of the
//!    bay to a bridge with red arches over the river mouth.
//! 7. Up a gorge to an arch dam; away from it up the hillside to a hairpin
//!    over the east coast, and back down straight onto its crest, narrow,
//!    the lake mirroring the hills on the left over a low parapet; down
//!    through the town and back to the seafront.
//!
//! The circuit is written in RON, `assets/circuits/turbo_bay.ron`; here is
//! only the function its file names: its mountains.

use bevy::prelude::*;

use crate::geometry::{fbm, smoothstep};

/// Across the sea, a mountainous coast to the east and the north.
pub(super) fn mountains(point: Vec2) -> f32 {
    const CENTER: Vec2 = Vec2::new(280.0, -300.0);
    let away = point - CENTER;
    let distance = away.length();
    // Only to the east and the north; to the west and the south, open sea.
    let facing = smoothstep(
        -0.2,
        0.5,
        away.normalize_or_zero().dot(Vec2::new(0.6, -0.8)),
    );
    smoothstep(1700.0, 2600.0, distance) * facing * 240.0 * (0.35 + 0.65 * fbm(point / 280.0, 4))
}
