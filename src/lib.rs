//! # wfmash-rs: Rust Bindings for wfmash Whole-Genome Aligner
//!
//! This library provides Rust bindings for the wfmash whole-genome alignment tool,
//! enabling all-vs-all and pairwise genome alignment with PAF output.
//!
//! ## Overview
//!
//! wfmash-rs wraps the wfmash binary (either vendored or system-installed) and
//! provides a safe Rust API for:
//! - All-vs-all whole-genome alignment (self-alignment)
//! - Pairwise genome alignment (query vs target)
//! - Configurable alignment parameters via builder pattern
//!
//! ## Example Usage
//!
//! ```no_run
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//! use wfmash_rs::{Wfmash, Config};
//! use std::path::Path;
//!
//! let config = Config::builder()
//!     .num_threads(8)
//!     .segment_length(5000)
//!     .build();
//!
//! let wfmash = Wfmash::new(config)?;
//!
//! // Self-alignment (all-vs-all)
//! let paf = wfmash.align_self(Path::new("genomes.fa.gz"))?;
//!
//! // Pairwise alignment
//! let paf = wfmash.align_files(
//!     Path::new("target.fa"),
//!     Path::new("query.fa"),
//! )?;
//! # Ok(())
//! # }
//! ```

mod binary_finder;
pub mod config;
pub mod error;
pub mod orchestrator;

pub use config::Config;
pub use error::WfmashError;
pub use orchestrator::WfmashOrchestrator;

use error::Result;
use std::path::Path;
use tempfile::NamedTempFile;

/// High-level interface to the wfmash aligner.
///
/// This struct wraps the orchestrator and provides a simple API
/// for running wfmash alignments.
pub struct Wfmash {
    orchestrator: WfmashOrchestrator,
}

impl Wfmash {
    /// Creates a new Wfmash instance with the given configuration.
    ///
    /// Verifies that the wfmash binary can be found.
    pub fn new(config: Config) -> Result<Self> {
        // Verify binary can be found early
        binary_finder::find_wfmash()?;
        Ok(Wfmash {
            orchestrator: WfmashOrchestrator::new(config),
        })
    }

    /// Align query against target, returning PAF output as bytes.
    ///
    /// # Arguments
    /// * `target` - Path to target FASTA file
    /// * `query` - Path to query FASTA file
    pub fn align_files(&self, target: &Path, query: &Path) -> Result<Vec<u8>> {
        self.orchestrator.align(target, query)
    }

    /// Self-alignment (all-vs-all) on a single FASTA file.
    ///
    /// # Arguments
    /// * `input` - Path to FASTA file
    pub fn align_self(&self, input: &Path) -> Result<Vec<u8>> {
        self.orchestrator.align_self(input)
    }

    /// Align query against target, writing PAF to a temporary file.
    ///
    /// Returns a NamedTempFile that will be deleted when dropped.
    pub fn align_to_temp_paf(&self, target: &Path, query: &Path) -> Result<NamedTempFile> {
        self.orchestrator.align_to_temp_paf(target, query)
    }

    /// Self-align and write PAF to a temporary file.
    pub fn align_self_to_temp_paf(&self, input: &Path) -> Result<NamedTempFile> {
        self.orchestrator.align_self_to_temp_paf(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .num_threads(4)
            .segment_length(5000)
            .map_pct_identity("90")
            .build();

        assert_eq!(config.num_threads, 4);
        assert_eq!(config.segment_length, Some(5000));
        assert_eq!(config.map_pct_identity, Some("90".to_string()));
    }

    #[test]
    fn test_config_to_args() {
        let config = Config::builder()
            .num_threads(8)
            .segment_length(10000)
            .self_mappings(true)
            .lower_triangular(true)
            .build();

        let args = config.to_args();
        assert!(args.contains(&"-t8".to_string()));
        assert!(args.contains(&"-s10000".to_string()));
        assert!(args.contains(&"-X".to_string()));
        assert!(args.contains(&"-L".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.num_threads > 0);
        assert!(!config.self_mappings);
        assert!(!config.lower_triangular);
        assert!(!config.mapping_only);
        assert!(config.map_pct_identity.is_none());
    }
}
