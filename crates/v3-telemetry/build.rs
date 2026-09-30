//! Resolves the build revision every telemetry resource carries.
//!
//! `PETRI_BUILD_REVISION` overrides it. Otherwise it is `git rev-parse HEAD`,
//! with `-dirty` appended when tracked files differ from `HEAD`, or `unknown`
//! when no repository is visible. Cargo reruns this script when `HEAD` or the
//! checked-out ref moves, not when a tracked file changes, so the dirty flag can
//! be stale between rebuilds.

use std::path::PathBuf;
use std::process::Command;

const OVERRIDE: &str = "PETRI_BUILD_REVISION";

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned())
}

/// The absolute path `git rev-parse --git-path name` resolves to; it follows a
/// linked worktree's `.git` file. Without `--path-format=absolute` Git answers
/// relative to the working directory (`../../.git/HEAD` from this crate).
fn git_path(name: &str) -> Option<PathBuf> {
    git(&["rev-parse", "--path-format=absolute", "--git-path", name]).map(PathBuf::from)
}

/// Tells Cargo to rerun when `name` changes. A ref with no loose file lives
/// in `packed-refs`; watching a missing path would rerun on every build.
fn watch_git_path(name: &str) {
    let watched = match git_path(name) {
        Some(path) if path.exists() => Some(path),
        _ => git_path("packed-refs").filter(|path| path.exists()),
    };
    if let Some(path) = watched {
        println!("cargo::rerun-if-changed={}", path.display());
    }
}

fn revision() -> String {
    let Some(head) = git(&["rev-parse", "HEAD"]).filter(|head| head.len() == 40) else {
        return "unknown".to_owned();
    };
    watch_git_path("HEAD");
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watch_git_path(&reference);
    }
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|status| !status.is_empty());
    if dirty {
        format!("{head}-dirty")
    } else {
        head
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed={OVERRIDE}");
    let revision = match std::env::var(OVERRIDE) {
        Ok(value) if !value.trim().is_empty() => value.trim().to_owned(),
        _ => revision(),
    };
    println!("cargo::rustc-env=PETRI_BUILD_REVISION_RESOLVED={revision}");
}
