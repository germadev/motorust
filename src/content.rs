//! The content pack: the game's files in `assets/` that are loaded through
//! Bevy's asset server, from the asset source [`SOURCE`]
//! (`content://music/bahia.mp3` is `assets/music/bahia.mp3`).
//!
//! Where they are read from depends on the build:
//!
//! - **A desktop development build** (`cargo run`, `cargo test`) reads them
//!   from the disk, from this repository's `assets/`, so a file can be
//!   changed with no compiling (and the pack's files are not built in, which
//!   keeps such builds quicker to link).
//! - **A release build, and every web build**, has them built into the
//!   executable: the files listed in `assets/pack.txt` are embedded by
//!   `build.rs` and served from memory, so the game stays a single file.
//!
//! Either way the game asks for them by the same path. This is step 3 of
//! the engine's split from the game (`ROADMAP.md`): the engine will ship its
//! open demo content as a pack of its own, and the game add its pack over
//! it. So far the music goes through it; the circuits, the roster, the
//! textures and the models are still built into the code.

use bevy::asset::io::AssetSourceBuilder;
use bevy::prelude::*;

/// The asset source the pack is read from.
pub const SOURCE: &str = "content";

/// The path the asset server knows a file of the pack by: `path` is where it
/// is in `assets/` (`music/bahia.mp3`).
pub fn path(file: &str) -> String {
    format!("{SOURCE}://{file}")
}

/// Registers the pack's asset source. It must be added before Bevy's
/// `AssetPlugin` (`DefaultPlugins`), which builds the sources it finds.
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(SOURCE, source());
    }
}

/// Where the pack is read from in this build.
fn source() -> AssetSourceBuilder {
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    {
        from_disk()
    }
    #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
    {
        from_memory(built_in::FILES)
    }
}

/// The files of `assets/`, read from the disk.
#[cfg(any(test, all(debug_assertions, not(target_arch = "wasm32"))))]
fn from_disk() -> AssetSourceBuilder {
    use bevy::asset::io::file::FileAssetReader;

    AssetSourceBuilder::new(|| {
        Box::new(FileAssetReader::new(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
        ))
    })
}

/// The `files` built into the game, each by its path in `assets/` with its
/// bytes, served from memory.
#[cfg(any(test, not(all(debug_assertions, not(target_arch = "wasm32")))))]
fn from_memory(files: &[(&str, &'static [u8])]) -> AssetSourceBuilder {
    use bevy::asset::io::memory::{Dir, MemoryAssetReader};

    let root = Dir::default();
    for &(file, bytes) in files {
        root.insert_asset(std::path::Path::new(file), bytes);
    }
    AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: root.clone() }))
}

/// The pack's files, each by its path in `assets/` with its bytes, as
/// `build.rs` lists them from `assets/pack.txt`.
#[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
mod built_in {
    include!(concat!(env!("OUT_DIR"), "/pack.rs"));
}

/// The files `assets/pack.txt` lists, each a path in `assets/`: what a
/// release build has built in.
#[cfg(test)]
pub fn listed() -> Vec<&'static str> {
    include_str!("../assets/pack.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// A file of the pack, read from the disk as the tests read it.
#[cfg(test)]
pub fn read(file: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pack_lists_files_that_are_there_once_each() {
        let files = listed();
        assert!(!files.is_empty());
        for (index, file) in files.iter().enumerate() {
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("assets")
                    .join(file)
                    .is_file(),
                "{file} is not in assets/"
            );
            assert!(!files[..index].contains(file), "{file} twice");
        }
    }

    #[test]
    fn a_file_of_the_pack_is_known_by_its_source() {
        assert_eq!(path("music/bahia.mp3"), "content://music/bahia.mp3");
    }

    /// A song loaded through the asset server from `source`, as the game
    /// loads it.
    fn load_song(source: AssetSourceBuilder) -> Vec<u8> {
        use crate::sound::music::{SongLoader, SongSource};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .register_asset_source(SOURCE, source)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<SongSource>()
            .init_asset_loader::<SongLoader>();
        let handle: Handle<SongSource> = app
            .world()
            .resource::<AssetServer>()
            .load(path("music/bahia.mp3"));
        for _ in 0..1000 {
            app.update();
            if let Some(song) = app.world().resource::<Assets<SongSource>>().get(&handle) {
                return song.bytes().to_vec();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the song did not load");
    }

    #[test]
    fn a_song_loads_from_the_disk_and_from_memory_alike() {
        let mp3 = read("music/bahia.mp3");
        assert_eq!(load_song(from_disk()), mp3, "from the disk");
        // As a release build serves it, built in.
        let built_in: &'static [u8] = Vec::leak(mp3.clone());
        assert_eq!(
            load_song(from_memory(&[("music/bahia.mp3", built_in)])),
            mp3,
            "from memory"
        );
    }
}
