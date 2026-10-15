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
