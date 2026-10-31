use std::path::Path;
use std::process::Command;

fn main() {
    // The commit the game is built from, for its logs (`MOTORUST_COMMIT`):
    // its short hash, with "-dirty" if files it tracks were changed and not
    // committed; "unknown" where there is no git (a copy of the sources).
    let commit = git(&["rev-parse", "--short=9", "HEAD"]).map_or_else(
        || "unknown".to_string(),
        |hash| {
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|changes| !changes.is_empty());
            if dirty { format!("{hash}-dirty") } else { hash }
        },
    );
    println!("cargo:rustc-env=MOTORUST_COMMIT={commit}");
    // Told again whenever a commit is made, the branch changes or the
    // sources do (which leave it dirty or not).
    for path in [
        ".git/HEAD",
        ".git/index",
        ".git/refs",
        "src",
        "assets",
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
    ] {
        if Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }

    pack();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        let _ = res.compile();
    }
}

/// What `git` says to `args`, trimmed, if it ran and said it.
fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Writes `pack.rs` in `OUT_DIR`: the content pack's files, those
/// `assets/pack.txt` lists, each by its path in `assets/` with its bytes
/// built in (`include_bytes!`), for `src/content.rs` to serve from memory in
/// a release build and on the web.
fn pack() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let list = std::fs::read_to_string(root.join("pack.txt")).expect("assets/pack.txt");
    let mut code = String::from("pub const FILES: &[(&str, &[u8])] = &[\n");
    for file in list
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let path = root.join(file);
        assert!(path.is_file(), "assets/pack.txt: no file assets/{file}");
        code += &format!(
            "    ({file:?}, include_bytes!({:?})),\n",
            path.display().to_string()
        );
    }
    code += "];\n";
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("pack.rs");
    std::fs::write(out, code).expect("pack.rs written");
}
