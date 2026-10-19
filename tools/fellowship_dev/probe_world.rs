//! A temporary probe (not committed): dumps Fellowship Road's centre line
//! and its ground for placing the props.

#[test]
#[ignore = "a probe"]
fn probe_dump() {
    use crate::track::Track;
    use crate::world::terrain::Terrain;
    use bevy::prelude::Vec2;
    use std::io::Write;
    let circuit = crate::circuits::CIRCUITS
        .iter()
        .find(|c| c.name == "Fellowship Road")
        .unwrap();
    let track = Track::new(&circuit.layout);
    let terrain = Terrain::new(&track, &circuit.scenery);
    let mut line = std::fs::File::create("/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/out/line.tsv").unwrap();
    for index in 0..track.sample_count() {
        let (p, heading) = track.pose_at(index);
        let road = track.surface_point(index, 0.0).y;
        writeln!(
            line,
            "{}\t{:.2}\t{:.2}\t{:.4}\t{:.2}\t{:.5}\t{:.2}",
            index,
            p.x,
            p.y,
            heading,
            road,
            track.curvature(index),
            track.wall_at(index)
        )
        .unwrap();
    }
    // The ground every 4 m over the whole map, and the nearest road.
    let mut grid = std::fs::File::create("/tmp/claude-0/-home-user-motorust/ee9612d8-e010-51d5-a057-47827c3b3e2b/scratchpad/out/ground.bin").unwrap();
    let (x0, x1, z0, z1) = (-500.0f32, 2300.0f32, -1000.0f32, 1100.0f32);
    let mut bytes = Vec::new();
    let mut z = z0;
    while z <= z1 {
        let mut x = x0;
        while x <= x1 {
            let at = Vec2::new(x, z);
            bytes.extend_from_slice(&terrain.height(at).to_le_bytes());
            bytes.extend_from_slice(&track.nearest(at).distance.to_le_bytes());
            x += 4.0;
        }
        z += 4.0;
    }
    grid.write_all(&bytes).unwrap();
    println!("lap {} steps {}", track.lap_length(), track.sample_count());
}
