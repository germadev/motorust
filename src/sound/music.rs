//! Music: each track has its own song ([`crate::circuits::Song`]), which
//! loops for the whole race. It plays well below the engines, so the bike
//! (revs, gear changes, tyres, impacts) is always heard over it: every song
//! is turned up or down to the same loudness, [`LOUDNESS`], about 13 dB under
//! the player's engine flat out, and still under it off the throttle.
//!
//! It fades in when the race starts and out when leaving it, drops in the
//! pause menu, and M mutes it with everything else.

use std::io::Cursor;

use bevy::audio::{AudioSinkPlayback, Decodable, Volume};
use bevy::prelude::*;

use super::{MASTER, SoundControls, music_gain};
use crate::circuits::{CIRCUITS, CurrentCircuit, Song};
use crate::menu::GameState;
use crate::pause::Pause;
use crate::settings::{DEFAULT_VOLUME, Settings};

/// Loudness every song plays at (LUFS), with the sound on. The player's
/// engine with the wind, measured the same way, is about -11 LUFS flat out,
/// -15 at half throttle, -18 off the throttle at speed and -30 idling on the
/// grid. The songs come mastered at -11 to -13 LUFS, as loud as the engine
/// flat out: they are turned down by 11 to 13 dB.
const LOUDNESS: f32 = -24.0;
/// Fader in the pause menu: about 6 dB lower.
const PAUSED: f32 = 0.5;
/// Time to fade in from silence to full volume, and out (s).
const FADE_IN: f32 = 2.0;
const FADE_OUT: f32 = 1.0;

/// A song as a Bevy audio source. It loops by decoding the MP3 again from the
/// start: Bevy's own looping keeps every decoded sample in memory (about
/// 60 MB for a 3-minute song).
#[derive(Asset, TypePath, Clone)]
pub struct SongSource {
    mp3: &'static [u8],
}

impl Decodable for SongSource {
    type Decoder = rodio::decoder::LoopedDecoder<Cursor<&'static [u8]>>;

    fn decoder(&self) -> Self::Decoder {
        rodio::Decoder::builder()
            .with_data(Cursor::new(self.mp3))
            .with_byte_len(self.mp3.len() as u64)
            .with_hint("mp3")
            .with_gapless(true)
            .build_looped()
            .expect("the tracks' songs are valid MP3")
    }
}

/// A track's song while it plays.
#[derive(Component)]
pub struct Music {
    /// Index of its track in [`CIRCUITS`].
    circuit: usize,
    /// From 0 (silent) to 1 (full volume).
    fader: f32,
}

/// How much a song is turned up or down to play at [`LOUDNESS`].
fn gain(song: &Song) -> f32 {
    10f32.powf((LOUDNESS - song.loudness) / 20.0)
}

/// Where the fader goes: up while racing on the song's track (lower in the
/// pause menu), down to silence otherwise.
fn fader_target(racing: bool, paused: bool) -> f32 {
    match (racing, paused) {
        (false, _) => 0.0,
        (true, true) => PAUSED,
        (true, false) => 1.0,
    }
}

/// Moves the fader towards `target`, at the speed of [`FADE_IN`] going up and
/// [`FADE_OUT`] going down.
fn fade(fader: f32, target: f32, dt: f32) -> f32 {
    if target > fader {
        (fader + dt / FADE_IN).min(target)
    } else {
        (fader - dt / FADE_OUT).max(target)
    }
}

/// Starts the track's song when the race starts, moves its fader and lets it
/// go once it has faded out. Real time, which keeps running in the pause menu.
#[allow(clippy::too_many_arguments)]
pub fn play_music(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    circuit: Res<CurrentCircuit>,
    pause: Res<Pause>,
    controls: Res<SoundControls>,
    settings: Option<Res<Settings>>,
    mut songs: ResMut<Assets<SongSource>>,
    mut playing: Query<(Entity, &mut Music, Option<&mut AudioSink>)>,
) {
    let dt = time.delta_secs();
    let racing = (*state.get() == GameState::Race).then_some(circuit.0);
    // M mutes the music too.
    let master = controls.0.master.get() / MASTER;
    let music_factor = settings
        .as_ref()
        .map_or(music_gain(DEFAULT_VOLUME), |settings| {
            if settings.music_enabled {
                music_gain(settings.music_volume)
            } else {
                0.0
            }
        });
    let mut started = false;
    for (entity, mut music, sink) in &mut playing {
        let current = racing == Some(music.circuit);
        started |= current;
        music.fader = fade(music.fader, fader_target(current, pause.is_open()), dt);
        if music.fader == 0.0 && !current {
            commands.entity(entity).despawn();
        } else if let Some(mut sink) = sink {
            let song = &CIRCUITS[music.circuit].song;
            sink.set_volume(Volume::Linear(
                music.fader * gain(song) * master * music_factor,
            ));
        }
    }
    if let Some(index) = racing
        && !started
    {
        let source = songs.add(SongSource {
            mp3: CIRCUITS[index].song.mp3,
        });
        commands.spawn((
            AudioPlayer(source),
            PlaybackSettings::ONCE.with_volume(Volume::Linear(0.0)),
            Music {
                circuit: index,
                fader: 0.0,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use bevy::audio::AddAudioSource;
    use bevy::time::TimeUpdateStrategy;
    use rodio::Source;

    use super::super::synth::{Controls, Mixer, SAMPLE_RATE, pan};
    use super::*;

    /// Root mean square of the samples, in dB.
    fn rms_db(samples: &[f32]) -> f32 {
        let power = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        10.0 * power.log10()
    }

    #[test]
    fn every_song_is_turned_to_the_same_loudness() {
        for circuit in &CIRCUITS {
            let song = &circuit.song;
            let played = song.loudness + 20.0 * gain(song).log10();
            assert!((played - LOUDNESS).abs() < 1e-3, "{}", circuit.name_en);
            assert!(gain(song) < 1.0, "{}: turned down", circuit.name_en);
        }
    }

    #[test]
    fn songs_decode_and_loop_back_to_the_start() {
        for circuit in &CIRCUITS {
            let source = SongSource {
                mp3: circuit.song.mp3,
            };
            let mut looped = source.decoder();
            assert_eq!(looped.channels().get(), 2, "{}", circuit.name_en);
            assert_eq!(looped.sample_rate().get(), 44_100, "{}", circuit.name_en);

            // The song once through, then the looped one past its end.
            let once = rodio::Decoder::builder()
                .with_data(Cursor::new(circuit.song.mp3))
                .with_byte_len(circuit.song.mp3.len() as u64)
                .with_hint("mp3")
                .with_gapless(true)
                .build()
                .unwrap();
            let length = once.count();
            assert!(
                length > 44_100 * 2 * 60,
                "{}: over a minute",
                circuit.name_en
            );
            let start: Vec<f32> = looped.by_ref().take(44_100 * 2).collect();
            assert!(
                start.iter().any(|s| s.abs() > 0.01),
                "{}: not silent",
                circuit.name_en
            );
            let again: Vec<f32> = looped
                .by_ref()
                .skip(length - start.len())
                .take(start.len())
                .collect();
            assert_eq!(again, start, "{}: starts over", circuit.name_en);
        }
    }

    #[test]
    fn the_music_stays_well_under_the_player_s_engine() {
        // The player's engine flat out, in the middle, with the wind, as
        // `place_sounds` and `player_sounds` leave it.
        let controls = Arc::new(Controls::default());
        controls.master.set(MASTER);
        let engine = &controls.engines[0];
        engine.rpm.set(14_000.0);
        engine.load.set(1.0);
        let (left, right) = pan(1.0, 0.0);
        engine.left.set(left);
        engine.right.set(right);
        controls.wind.set(0.4);
        let mix: Vec<f32> = Mixer::new(controls)
            .skip(SAMPLE_RATE as usize)
            .take(SAMPLE_RATE as usize * 2 * 4)
            .collect();
        let engine = rms_db(&mix);

        for circuit in &CIRCUITS {
            let song = &circuit.song;
            let samples: Vec<f32> = SongSource { mp3: song.mp3 }
                .decoder()
                .take(SAMPLE_RATE as usize * 2 * 60)
                .map(|s| s * gain(song))
                .collect();
            // Its loudest second.
            let loudest = samples
                .chunks(SAMPLE_RATE as usize * 2)
                .map(rms_db)
                .fold(f32::MIN, f32::max);
            assert!(
                loudest < engine - 10.0,
                "{}: {loudest:.1} dB against the engine's {engine:.1} dB",
                circuit.name_en
            );
        }
    }

    #[test]
    fn fades_in_dips_in_the_pause_and_fades_out() {
        let mut fader = 0.0;
        let mut time = 0.0;
        while fader < 1.0 {
            fader = fade(fader, fader_target(true, false), 0.1);
            time += 0.1;
        }
        assert!((time - FADE_IN).abs() < 0.15, "fades in: {time}");
        for _ in 0..20 {
            fader = fade(fader, fader_target(true, true), 0.1);
        }
        assert_eq!(fader, PAUSED, "lower in the pause menu");
        let mut time = 0.0;
        while fader > 0.0 {
            fader = fade(fader, fader_target(false, false), 0.1);
            time += 0.1;
        }
        assert!(time <= FADE_OUT + 0.05, "fades out: {time}");
    }

    fn music_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(bevy::audio::AudioPlugin::default())
            .add_audio_source::<SongSource>()
            .init_state::<GameState>()
            .init_resource::<Pause>()
            .insert_resource(CurrentCircuit(2))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .add_systems(Update, play_music);
        let controls = Arc::new(Controls::default());
        controls.master.set(MASTER);
        app.insert_resource(SoundControls(controls));
        app
    }

    fn songs_playing(app: &mut App) -> Vec<(usize, f32)> {
        let mut query = app.world_mut().query::<&Music>();
        query
            .iter(app.world())
            .map(|music| (music.circuit, music.fader))
            .collect()
    }

    #[test]
    fn the_track_s_song_plays_during_the_race_only() {
        let mut app = music_app();
        app.update();
        assert!(songs_playing(&mut app).is_empty(), "not in the menus");

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Race);
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(songs_playing(&mut app), [(2, 1.0)], "the track's song");

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Welcome);
        app.update();
        app.update();
        let fading = songs_playing(&mut app);
        assert!(fading.len() == 1 && fading[0].1 < 1.0, "fades out");
        for _ in 0..15 {
            app.update();
        }
        assert!(songs_playing(&mut app).is_empty(), "and stops");
    }
}
