//! Utility for finding the wfmash binary at runtime.
//!
//! Search order:
//! 1. Same directory as current executable (cargo install)
//! 2. $CARGO_HOME/lib/wfmash-rs/ (for dependent packages)
//! 3. OUT_DIR from build.rs (development builds)
//! 4. $WFMASH_BIN or $WFMASH_BIN_DIR env var
//! 5. Cargo build directories (development)
//! 6. System PATH via which::which

use crate::error::{Result, WfmashError};
use std::path::PathBuf;

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
        if let Some(exe_dir) = exe_path.parent() {
            let build_dir = exe_dir.join("build");
            if let Ok(entries) = std::fs::read_dir(&build_dir) {
                for entry in entries.flatten() {
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("wfmash-rs-")
                    {
                        let binary = entry.path().join("out").join(name);
                        if binary.exists() {
                            return Ok(binary);
                        }
                    }
                }
            }
        }
    }

    // Also try target directories (running from project root)
    for profile in &["debug", "release"] {
        let build_dir = PathBuf::from(format!("target/{profile}/build"));
        if let Ok(entries) = std::fs::read_dir(&build_dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("wfmash-rs-")
                {
                    let binary = entry.path().join("out").join(name);
                    if binary.exists() {
                        return Ok(binary);
                    }
                }
            }
        }
    }

    // 6. System PATH
    if let Ok(path) = which::which(name) {
        return Ok(path);
    }

    Err(WfmashError::BinaryNotFound)
}
