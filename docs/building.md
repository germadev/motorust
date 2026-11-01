# Building and releasing

How to build and run motorust on the desktop and in the browser, and how the GitHub Actions workflows test, package and publish it. For the code layout see [architecture.md](architecture.md); for how to play see [gameplay.md](gameplay.md).

## Requirements

- **Rust 1.95 or later** (install it with [rustup](https://rustup.rs)). The crate uses edition 2024 and declares `rust-version = "1.95"`.
- **`make`**, for the shortcuts below. They are thin wrappers around `cargo`, so without `make` (for example on Windows) you can run the `cargo` commands directly.
- **Linux only:** the system libraries Bevy needs (a C++ compiler, `pkg-config`, ALSA, udev, X11 and Wayland). `make install` installs them with `apt`, `dnf` or `pacman`, using `sudo` unless you are root. On other distributions, install the packages listed in [Bevy's Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md).
- **macOS:** nothing else; `make install` only downloads the crates.

## Make targets

```sh
make install   # system dependencies (Linux: apt, dnf or pacman) and crates
make up        # runs the game in development mode
make build     # optimised build in target/release/motorust
make app       # (macOS) the optimised build as target/release/Motorust.app
make test      # runs the tests
make bench     # rides every track alone under each graphics setting; results in bench-results/
make bench.push  # commits bench-results/ and pushes it to the branch
```

| Target | Runs | What it does |
|---|---|---|
| `make` / `make help` | | Lists the targets. |
| `make install` | Linux packages, then `cargo fetch` | Installs the system dependencies (Linux only) and downloads the crates. Fails early if `cargo` is missing. |
| `make up` | `cargo run` | Builds and starts the game with the `dev` profile. |
| `make build` | `cargo build --release` | Optimised executable in `target/release/motorust` (`motorust.exe` on Windows). |
| `make app` | `cargo build --release`, then `tools/make-app.sh` | macOS only: `target/release/Motorust.app` around the optimised executable (with the icon and `assets/Info.plist`), to open from Finder or keep in the Dock. Its Info.plist says it is a racing game (`LSApplicationCategoryType` `public.app-category.racing-games`, `LSSupportsGameMode`), so macOS (Sonoma and later) turns its Game Mode on when the game goes fullscreen, which a bare executable run from the terminal never gets. The CI's `.app` is made by the same script. Launched from Finder, the game runs from `/`: the benchmark is for `make bench`. |
| `make test` | `cargo test` | Runs the tests (see [architecture.md](architecture.md#tests)). |
| `make bench` | `cargo run --release -- --bench` (on macOS under `caffeinate -dis`, so the display does not sleep and take the game's window with it) | The benchmark: the game rides every track on its own, in free ride on autopilot, with one player and with two, under each graphics setting worth comparing (shadows high, medium and low; the ground out of the shadows; MSAA 4× and 2× (where the GPU has two samples), SMAA, FXAA or none; lighter far trees; the world drawn at 70 % and 50 % of the window's resolution; the relief off, at speed, with parallax and on the road; the lawn's tiling; less detail for two players, and that at 70 %, without relief and with the ground out of the shadows), with the frame rate unlimited. Each track is ridden all round, once with one player and once with two, the settings taking turns of 2 s (after half a second to settle) all over the lap, each round in another order, so that every setting is measured on the same roads as the rest; it rides on until it has gone all round and each one has been measured for 10 s (`MOTORUST_BENCH_SECONDS`); each lap begins with every setting tried once, unmeasured, while the shaders compile. On the tracks in `MOTORUST_BENCH_TRACKS` (as `0,2`) or all of them. It says what it is doing in the terminal (the machine: system, CPU, memory, GPU, window, the game's version and commit; where the results go; each lap as it starts, every quarter of it, and as it ends each setting's frame rate, slowest 1 %, frame times, simulation time and CPU, and where on the lap it was slowest and fastest) and on the screen, under the frame rate (the lap, the setting on, the pixels the world is drawn at, how far round). The results go to `bench-results/` in the folder it is run from (the repository, with `make bench`), one file for each run named after when (UTC), the system, the CPU and the GPU (`2026-10-16_1830_macos-26-3-1_apple-m1-pro.txt`), headed with the machine and the commit, each lap's lines written as soon as it is measured, with the run's performance log beside it (`.perf.log`): runs on several machines sit side by side, to commit and compare. Esc stops it, keeping the laps it has measured (not the one under way), and prints the table so far (closing the window or Ctrl+C keep it too). The player's settings are used as they are and never saved over. |
| `make bench.push` | `git add bench-results`, `git commit -- bench-results`, `git pull --rebase --autostash`, `git push` | Commits the benchmark's results (only `bench-results/`, whatever else is changed and not committed) as "bench: results from <machine>", brings in first what other machines have pushed to the branch (anything not committed is put aside and back), and pushes. With nothing new it says so and makes no commit. |

### Build times and profiles

The first build takes several minutes because it compiles all of Bevy; later builds are quick. `make up` runs smoothly even though it is a development build, because `Cargo.toml` tunes the profiles:

| Profile | Settings | Why |
|---|---|---|
| `dev` | `opt-level = 1` for motorust, `opt-level = 3` for every dependency | Our code compiles fast, while Bevy stays fast at runtime (unoptimised Bevy runs at a few FPS). |
| `release` | `codegen-units = 1`, `lto = "thin"` | Maximum performance in exchange for slower builds. |

To compile even faster while developing, link Bevy dynamically:

```sh
cargo run --features bevy/dynamic_linking
```

## Starting part way round the lap

To look at the scenery of a stretch without riding all the way to it, start
the game with the distance you want, in metres along the lap:

```sh
cargo run -- --from 3700        # or MOTORUST_FROM=3700 make up
```

The whole grid stands there instead of behind the finish line, in whatever
track and mode are chosen (the distance wraps round, so 3700 on a 1.6 km
track is 500 m). It is for development only: the lap and the times are
counted from the finish line as ever, so the first lap is short and its
time means nothing. See `src/debug_start.rs`.

The release executable is self-contained: every file the game uses (the circuits, the bikes, the 3D models, the textures, the font, the art and the music), in the content packs (the engine's, `motorust-engine/assets/pack.txt`, and the game's over it, `assets/pack.txt`; see `src/content.rs`), is built in by `build.rs`, so you do not need to copy the `assets/` folders to play (see [models.md](models.md)). A desktop development build (`cargo run`) reads the packs from the disk instead, and builds none of them in. `MOTORUST_ENGINE_ONLY=1 cargo test -- --skip game::` runs the engine's tests on the engine's pack alone, as CI does. On Windows the release build opens without a console window (`windows_subsystem = "windows"` in `src/main.rs`); debug builds keep it so log output stays visible.

## Web version

The game also compiles to WebAssembly and runs in the browser with WebGL2 at **<https://motorust.germade.dev>**.

### Building it locally

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --locked --version 0.2.129   # the wasm-bindgen version in Cargo.lock
web/build.sh                                                 # writes the page to dist/web
python3 -m http.server -d dist/web                           # then open http://localhost:8000
```

`wasm-bindgen-cli` must have exactly the same version as the `wasm-bindgen` crate in `Cargo.lock`, or the generated bindings will not match the module. Check it with:

```sh
grep -A1 '^name = "wasm-bindgen"$' Cargo.lock
```

`web/build.sh [OUTPUT_DIR]` (default `dist/web`):

1. Runs `cargo build --release --target wasm32-unknown-unknown`.
2. Runs `wasm-bindgen --target web`, dropping the name and producers sections (the function names take almost as much space as the code).
3. Copies `web/index.html` into the output folder with the size of the `.wasm` filled in (`WASM_BYTES`), so the progress bar works even when the server sends the file compressed without a length, and its SHA-256 (`WASM_SHA256`), which names the build (see [Caching the game](#caching-the-game)).
4. Copies `web/setup.sh` into the output folder with the version of the build filled in, so the installer for Linux and macOS downloads the same release the page is publishing (see [The installer for Linux and macOS](#the-installer-for-linux-and-macos)).

The result is three files for the game (`index.html`, `motorust.js` and `motorust_bg.wasm`) plus the installer (`setup.sh`).

### The page

`web/index.html` gives the game the whole window and shows a screen on top of it with the title, a "Jugar online" (*Play online*) button and the desktop downloads. Nothing of the game is downloaded when the page opens: the button says how much it will download (or that it is already kept, see below), and only once it is pressed (or Enter or Space) does the page fetch the `.wasm` and the JavaScript glue (`motorust.js`, imported then). It shows download progress ("Descargando el juego…", *Downloading the game…*), then "Preparando la pista…" (*Preparing the track…*) while the first frame builds the world. When the game is visible, `src/web.rs` sets `data-motorust="ready"` on the page and the loading screen fades out. If the browser has no WebGL2, or the game fails before it is shown, the page displays the error instead.

### Caching the game

The browser keeps the `.wasm` between visits, so reloading the tab starts the game without downloading it again:

- The page asks for `motorust_bg.wasm?v=<first 16 hex digits of its SHA-256>`. Each build has its own URL, so the server can let browsers keep the file for a year (`Cache-Control: immutable`, set in `functions/motorust_bg.wasm.js`) and no browser reuses an old build's `.wasm` with a new build's `motorust.js`.
- After downloading it, the page keeps a copy in [Cache Storage](https://developer.mozilla.org/docs/Web/API/CacheStorage) (cache `motorust-wasm`), but only if its SHA-256 matches the page's `WASM_SHA256`, and deletes the copies of older builds. On the next visit, once "Jugar online" is pressed, the game comes from there (the page says "Ya descargado: empieza al momento" beforehand), even after a hard reload, which skips the HTTP cache.
- Cache Storage only exists on https and `localhost`, and the browser may refuse it (some private windows, no disk space) or clear it to free space. Then the page just downloads the game as before.

This is not WasmFS: WasmFS is Emscripten's file system for programs built with Emscripten, while motorust is built with `wasm32-unknown-unknown` and `wasm-bindgen`, and keeps its settings in `localStorage`.

### What changes in the browser

- The game fills the page. Under Options, the resolution choices are "Toda la página" (*Whole page*) or "Pantalla completa" (*Full screen*). Cmd+Enter and Alt+Enter both work.
- There is no FPS limit (the browser decides when to draw frames) and no Quit option (close the tab instead).
- Sound starts on the first key press or click: browsers do not allow audio before the user interacts with the page.
- Gamepads show up only after you press a button with the page open, for the same reason.
- Settings are saved in the browser's `localStorage` (key `motorust.ajustes`) instead of `ajustes.ron`.
- The terrain is computed on a single thread (there are no threads on the web), so preparing a track takes a bit longer than on the desktop.
- The grass, flowers and petals end at once where they stop being drawn instead of fading out (`fading_out` in `src/world/detail.rs`). On WebGL2, Bevy 0.19 binds the table of distances its fading reads (a uniform buffer of 64 entries, 1024 bytes) with a minimum size of one entry (16 bytes); wgpu then refuses the pipeline of every mesh that fades (`pbr_opaque_mesh_pipeline`, group 0, binding 14) and the game quits before it shows.

### Size

Without the music, the `.wasm` file was measured at about **84 MB**, because the models, textures and font are embedded in it. That is about **33 MB gzipped**, which is roughly what the browser downloads. Ghost Island's props and Japanese trees have since added about 4.2 MB of models, and Fellowship Road's props about 8.8 MB more (mostly JPEG and vertex data, which compress little), not measured in a web build yet. The music (`assets/music/`, ten 192 kbps MP3s embedded: the six tracks' songs, three chill ones and the menus'; five more, "The Iron Paddock", "Orange Asphalt", "Crown of the Track", "Pursuit at the High Pass" and "Last Lap of the Quest", are kept there for tracks to come and embedded by none yet) adds about **42.3 MB**, which compression does not shrink: about **126 MB**, or **75 MB gzipped**, in all. The JavaScript glue is about 120 KB.

## Continuous integration

There are two workflows in `.github/workflows/`:

| Workflow | Runs on | What it does |
|---|---|---|
| [`build.yml`](#buildyml) | Push to `main`, every PR, manually, and called from `release.yml` | Format check, clippy, tests, and builds for four desktop targets and WebAssembly. |
| [`release.yml`](#releaseyml) | Manually only | Bumps the version, builds everything, deploys the web version to Cloudflare Pages and creates a GitHub release. |

In a PR, a new push cancels the previous run of the same workflow.

### `build.yml`

1. **Tests, format and clippy** (Ubuntu): `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (any warning fails the build) and `cargo test`.
2. **Builds**, once the tests pass, with `cargo build --release --target <target>`:

   | Build | Runner | Target |
   |---|---|---|
   | Linux x86_64 | `ubuntu-latest` | `x86_64-unknown-linux-gnu` |
   | Windows x86_64 | `windows-latest` | `x86_64-pc-windows-msvc` |
   | macOS Apple Silicon | `macos-latest` | `aarch64-apple-darwin` |
   | macOS Intel | `macos-latest` | `x86_64-apple-darwin` |
   | WebAssembly | `ubuntu-latest` | `wasm32-unknown-unknown` |

3. **Packaging:** each desktop build becomes `motorust-<version>-<target>.tar.gz` (a `.zip` on Windows) containing the executable, `README.md` and `CREDITS.md`. On macOS, both `motorust-<version>-<target>.tar.gz` (command-line binary) and `motorust-<version>-<target>-app.zip` (containing `Motorust.app` to launch directly from Finder without opening Terminal) are produced. The WebAssembly build becomes `motorust-<version>-wasm32-unknown-unknown.zip` containing the served files (`index.html`, `motorust.js`, `motorust_bg.wasm`), `README.md` and `CREDITS.md`. Each archive is uploaded as a run artifact named `motorust-<target>`, and the web folder is uploaded as `web-dist`. The version is the release version when `release.yml` calls the workflow (it is also written into `Cargo.toml`, so the executable carries it), and `dev-<short commit hash>` otherwise.

### `release.yml`

Started by hand from **Actions → Release → Run workflow**. Releases run one at a time, never in parallel.

1. Choose which part of the version to bump: `patch`, `minor` or `major`.
2. The workflow computes the new version from the latest `vX.Y.Z` tag. The repository's model releases (which hold the original 3D models, see [models.md](models.md)) are not `vX.Y.Z` tags, so they do not count. With no version tag yet, it starts from 0.0.0, so the first release is 0.0.1, 0.1.0 or 1.0.0. The run fails if the new tag already exists.
3. It builds every target with `build.yml` and that version.
4. It publishes the WebAssembly build (`web-dist`) to <https://motorust.germade.dev>. The `.wasm` is larger than the 25 MiB a Cloudflare Pages file may have, so it goes to the R2 bucket with the desktop downloads, and `functions/motorust_bg.wasm.js` serves it from there. Everything else goes to Cloudflare Pages, after the `.wasm`, so the new page never asks for a build that is not there yet.
5. It updates `Cargo.toml` and `Cargo.lock` with the new version and commits them to `main`.
6. It downloads the archives, writes their SHA-256 sums to `SHA256SUMS.txt` and creates the GitHub release `vX.Y.Z` ("Motorust X.Y.Z") on `main` with the archives, the sums and notes generated by GitHub.

## Running a downloaded build

Download an archive from the [releases page](https://github.com/germadev/motorust/releases) (or from a workflow run's artifacts) and extract it. To check it against the published sums:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
```

- **macOS:** the executables are not signed. The first time, open the game with right-click → Open, or remove the quarantine flag with `xattr -d com.apple.quarantine motorust` (or on `Motorust.app`). Double-clicking `Motorust.app` opens the game directly without launching Terminal. macOS knows it for a racing game and turns its Game Mode on in fullscreen. On macOS the game also asks, while it runs, not to be napped, for its timers on time and for neither the display nor the Mac to sleep (what `caffeinate` does).
- **Windows:** the release build opens without a console window.
- **Linux:** run `./motorust` from the extracted folder.

## The installer for Linux and macOS

`web/setup.sh` is published with the site at <https://motorust.germade.dev/setup.sh> and installs the game from the terminal:

```sh
curl -fsSL https://motorust.germade.dev/setup.sh | bash
wget -qO- https://motorust.germade.dev/setup.sh | bash
```

It downloads `motorust-<version>-<target>.tar.gz` from `/downloads/` (the Pages function serves it from the R2 bucket, and falls back to the GitHub release), checks it against `SHA256SUMS.txt`, leaves the executable in `~/motorust/motorust` and links it into a directory of `$PATH` (`~/.local/bin` when it is there, otherwise the first writable one) so the game runs with `motorust`. Run it again and it offers to update or uninstall. It also takes `--install`, `--update`, `--uninstall`, `--dir` and `--version`.

`web/build.sh` writes the version of the build into the script (the `__MOTORUST_VERSION__` placeholder), so the installer always downloads the release the site is publishing, and copies it to `dist/web/setup.sh`.
