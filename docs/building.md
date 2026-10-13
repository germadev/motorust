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
make test      # runs the tests
```

| Target | Runs | What it does |
|---|---|---|
| `make` / `make help` | | Lists the targets. |
| `make install` | Linux packages, then `cargo fetch` | Installs the system dependencies (Linux only) and downloads the crates. Fails early if `cargo` is missing. |
| `make up` | `cargo run` | Builds and starts the game with the `dev` profile. |
| `make build` | `cargo build --release` | Optimised executable in `target/release/motorust` (`motorust.exe` on Windows). |
| `make test` | `cargo test` | Runs the 229 tests (see [architecture.md](architecture.md#tests)). |

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

The executable is self-contained: the 3D models, textures and font are embedded with `include_bytes!`, so you do not need to copy the `assets/` folder to play (see [models.md](models.md)). On Windows the release build opens without a console window (`windows_subsystem = "windows"` in `src/main.rs`); debug builds keep it so log output stays visible.

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

The result is three files: `index.html`, `motorust.js` and `motorust_bg.wasm`.

### The page

`web/index.html` gives the game the whole window and shows a loading screen on top of it. The screen shows download progress ("Descargando el juego…", *Downloading the game…*), then "Preparando la pista…" (*Preparing the track…*) while the first frame builds the world. When the game is visible, `src/web.rs` sets `data-motorust="ready"` on the page and the loading screen fades out. If the browser has no WebGL2, or the game fails before it is shown, the page displays the error instead.

### Caching the game

The browser keeps the `.wasm` between visits, so reloading the tab starts the game without downloading it again:

- The page asks for `motorust_bg.wasm?v=<first 16 hex digits of its SHA-256>`. Each build has its own URL, so the server can let browsers keep the file for a year (`Cache-Control: immutable`, set in `functions/motorust_bg.wasm.js`) and no browser reuses an old build's `.wasm` with a new build's `motorust.js`.
- After downloading it, the page keeps a copy in [Cache Storage](https://developer.mozilla.org/docs/Web/API/CacheStorage) (cache `motorust-wasm`), but only if its SHA-256 matches the page's `WASM_SHA256`, and deletes the copies of older builds. On the next visit the game comes from there ("Listo, sin volver a descargarlo"), even after a hard reload, which skips the HTTP cache.
- Cache Storage only exists on https and `localhost`, and the browser may refuse it (some private windows, no disk space) or clear it to free space. Then the page just downloads the game as before.

This is not WasmFS: WasmFS is Emscripten's file system for programs built with Emscripten, while motorust is built with `wasm32-unknown-unknown` and `wasm-bindgen`, and keeps its settings in `localStorage`.

### What changes in the browser

- The game fills the page. Under Options, the resolution choices are "Toda la página" (*Whole page*) or "Pantalla completa" (*Full screen*). Cmd+Enter and Alt+Enter both work.
- There is no FPS limit (the browser decides when to draw frames) and no Quit option (close the tab instead).
- Sound starts on the first key press or click: browsers do not allow audio before the user interacts with the page.
- Gamepads show up only after you press a button with the page open, for the same reason.
- Settings are saved in the browser's `localStorage` (key `motorust.ajustes`) instead of `ajustes.ron`.
- The terrain is computed on a single thread (there are no threads on the web), so preparing a track takes a bit longer than on the desktop.

### Size

Without the music, the `.wasm` file is about **84 MB**, because the models, textures and font are embedded in it. That is about **33 MB gzipped**, which is roughly what the browser downloads. The tracks' music (`assets/music/`, three 192 kbps MP3s) adds about **12.4 MB**, which compression does not shrink: about **96 MB**, or **45 MB gzipped**, in all. The JavaScript glue is about 120 KB.

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

- **macOS:** the executables are not signed. The first time, open the game with right-click → Open, or remove the quarantine flag with `xattr -d com.apple.quarantine motorust` (or on `Motorust.app`). Double-clicking `Motorust.app` opens the game directly without launching Terminal.
- **Windows:** the release build opens without a console window.
- **Linux:** run `./motorust` from the extracted folder.
