//! Utility for finding the wfmash binary at runtime.
//!
//! Search order:
//! 1. Same directory as current executable (cargo install)
//! 2. $CARGO_HOME/lib/wfmash-rs/ (for dependent packages)
//! 3. OUT_DIR from build.rs (development builds)
//! 4. $WFMASH_BIN or $WFMASH_BIN_DIR env var
//! 5. Cargo build directories (development), both the
//!    `build/<crate>-<hash>/out` and `build/<crate>/<hash>/out` layouts
//!
//! The system PATH is intentionally NOT searched to avoid version
//! mismatches with a globally installed wfmash.

use crate::error::{Result, WfmashError};
use std::path::{Path, PathBuf};

const CRATE: &str = "wfmash-rs";

/// Find the wfmash binary.
pub fn find_wfmash() -> Result<PathBuf> {
    find_binary("wfmash")
}

/// Find a binary by name using the standard search order.
fn find_binary(name: &str) -> Result<PathBuf> {
    // 1. Same directory as the current executable (for cargo install)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let binary = exe_dir.join(name);
            if binary.exists() {
                return Ok(binary);
            }
        }
    }

    // 2. $CARGO_HOME/lib/wfmash-rs/
    let cargo_home = std::env::var("CARGO_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".cargo"))
        });

    if let Some(cargo_home) = cargo_home {
        let lib_dir = cargo_home.join("lib");
        for package in &["wfmash-rs", "sweepga", "wfmash"] {
            let binary = lib_dir.join(package).join(name);
            if binary.exists() {
                return Ok(binary);
            }
        }
    }

    // 3. OUT_DIR from build.rs (compile-time env var, only works during build)
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let path = PathBuf::from(out_dir).join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    // 4. WFMASH_BIN (explicit path) or WFMASH_BIN_DIR (directory containing binary)
    if let Ok(bin_path) = std::env::var("WFMASH_BIN") {
        let path = PathBuf::from(bin_path);
        if path.exists() {
            return Ok(path);
        }
    }
    if let Ok(bin_dir) = std::env::var("WFMASH_BIN_DIR") {
        let path = PathBuf::from(bin_dir).join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    // 5. Cargo build directories (development)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(binary) = exe_path
            .parent()
            .and_then(build_dir_above)
            .and_then(|b| newest_built(&b, CRATE, name))
        {
            return Ok(binary);
        }
    }

    // Also try target directories (running from project root)
    for profile in &["debug", "release"] {
        let build_dir = PathBuf::from(format!("target/{profile}/build"));
        if let Some(binary) = newest_built(&build_dir, CRATE, name) {
            return Ok(binary);
        }
    }

    Err(WfmashError::BinaryNotFound)
}

/// `out` dirs of `krate`, both cargo layouts.
fn out_dirs(build_dir: &Path, krate: &str) -> Vec<PathBuf> {
    let list = |d: &Path| std::fs::read_dir(d).into_iter().flatten().flatten();
    let mut outs = Vec::new();
    for entry in list(build_dir) {
        let file_name = entry.file_name();
        let dir = file_name.to_string_lossy();
        if dir == krate {
            // cargo >= 1.100: <crate>/<hash>/out
            outs.extend(list(&entry.path()).map(|h| h.path().join("out")));
        } else if dir.strip_prefix(krate).is_some_and(|r| r.starts_with('-')) {
            // cargo <= 1.99: <crate>-<hash>/out
            outs.push(entry.path().join("out"));
        }
    }
    outs
}

/// Most recently modified `name` under `build_dir`.
fn newest_built(build_dir: &Path, krate: &str, name: &str) -> Option<PathBuf> {
    out_dirs(build_dir, krate)
        .into_iter()
        .map(|out| out.join(name))
        .filter(|p| p.is_file())
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
}

/// Nearest `build` dir at or above `dir`.
fn build_dir_above(dir: &Path) -> Option<PathBuf> {
    dir.ancestors().find_map(|a| {
        if a.file_name().is_some_and(|n| n == "build") {
            return Some(a.to_path_buf());
        }
        Some(a.join("build")).filter(|b| b.is_dir())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn touch(out: &Path, age_secs: u64) -> PathBuf {
        std::fs::create_dir_all(out).unwrap();
        let bin = out.join("wfmash");
        let file = std::fs::File::create(&bin).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(age_secs))
            .unwrap();
        bin
    }

    #[test]
    fn test_both_build_layouts() {
        let tmp = tempfile::tempdir().unwrap();
        let build = tmp.path().join("release").join("build");
        let find = || newest_built(&build, CRATE, "wfmash");
        assert_eq!(find(), None);

        // Other crates must not match
        touch(&build.join("wfmash-rsx-0a1b").join("out"), 0);
        touch(&build.join("other").join("0a1b").join("out"), 0);
        assert_eq!(find(), None);

        let old_out = build.join("wfmash-rs-0a1b").join("out");
        let old = touch(&old_out, 200);
        assert_eq!(find(), Some(old.clone()));

        let new_out = build.join("wfmash-rs").join("2c3d").join("out");
        let new = touch(&new_out, 100);
        assert_eq!(find(), Some(new.clone()));
        assert_eq!(out_dirs(&build, CRATE).len(), 2);

        // Newest wins in either layout
        touch(&old_out, 0);
        assert_eq!(find(), Some(old));
        std::fs::remove_dir_all(&old_out).unwrap();
        assert_eq!(find(), Some(new));

        // Executable beside or inside build
        let release = build.parent().unwrap();
        std::fs::create_dir_all(release.join("deps")).unwrap();
        for exe_dir in [release.to_path_buf(), release.join("deps"), new_out] {
            assert_eq!(build_dir_above(&exe_dir), Some(build.clone()));
        }
    }
}
