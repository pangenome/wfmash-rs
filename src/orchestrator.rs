//! Subprocess execution for wfmash.
//!
//! This module handles building and running the wfmash command-line invocation.

use crate::binary_finder::find_wfmash;
use crate::config::Config;
use crate::error::{Result, WfmashError};
use std::path::Path;
use tempfile::NamedTempFile;

/// Orchestrates wfmash subprocess execution.
pub struct WfmashOrchestrator {
    pub config: Config,
}

impl WfmashOrchestrator {
    /// Create a new orchestrator with the given config.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Align query against target, returning PAF output as bytes.
    ///
    /// Runs: `wfmash <target> <query> [options]`
    pub fn align(&self, target: &Path, query: &Path) -> Result<Vec<u8>> {
        self.validate_input(target)?;
        self.validate_input(query)?;

        let wfmash_bin = find_wfmash()?;
        let mut cmd = self.build_command(&wfmash_bin);

        cmd.arg(target).arg(query);

        eprintln!(
            "[wfmash] Aligning {} vs {}",
            target.display(),
            query.display()
        );
        self.run_command(cmd)
    }

    /// Self-alignment (single file against itself).
    ///
    /// Runs: `wfmash <input> [options]`
    /// Note: wfmash performs all-vs-all mapping when given a single file.
    pub fn align_self(&self, input: &Path) -> Result<Vec<u8>> {
        self.validate_input(input)?;

        let wfmash_bin = find_wfmash()?;
        let mut cmd = self.build_command(&wfmash_bin);

        cmd.arg(input);

        eprintln!("[wfmash] Self-aligning {}", input.display());
        self.run_command(cmd)
    }

    /// Align query against target, writing PAF to a temporary file.
    ///
    /// Returns the NamedTempFile containing PAF output.
    pub fn align_to_temp_paf(&self, target: &Path, query: &Path) -> Result<NamedTempFile> {
        self.validate_input(target)?;
        self.validate_input(query)?;

        let temp_file = self.create_temp_file()?;
        let wfmash_bin = find_wfmash()?;
        let mut cmd = self.build_command(&wfmash_bin);

        cmd.arg(target).arg(query);

        eprintln!(
            "[wfmash] Aligning {} vs {} -> {}",
            target.display(),
            query.display(),
            temp_file.path().display()
        );
        self.run_command_to_file(cmd, temp_file)
    }

    /// Self-align and write PAF to a temporary file.
    pub fn align_self_to_temp_paf(&self, input: &Path) -> Result<NamedTempFile> {
        self.validate_input(input)?;

        let temp_file = self.create_temp_file()?;
        let wfmash_bin = find_wfmash()?;
        let mut cmd = self.build_command(&wfmash_bin);

        cmd.arg(input);

        eprintln!(
            "[wfmash] Self-aligning {} -> {}",
            input.display(),
            temp_file.path().display()
        );
        self.run_command_to_file(cmd, temp_file)
    }

    /// Build the base wfmash command with config args.
    fn build_command(&self, wfmash_bin: &Path) -> std::process::Command {
        let mut cmd = std::process::Command::new(wfmash_bin);

        // Set TMPDIR if configured
        if let Some(ref temp_dir) = self.config.temp_dir {
            cmd.env("TMPDIR", temp_dir);
        }

        // Add all config-derived arguments
        for arg in self.config.to_args() {
            cmd.arg(arg);
        }

        cmd
    }

    /// Validate that an input file exists.
    fn validate_input(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(WfmashError::FileNotFound(path.to_path_buf()));
        }
        Ok(())
    }

    /// Create a temporary file for PAF output.
    fn create_temp_file(&self) -> Result<NamedTempFile> {
        let builder = tempfile::Builder::new().suffix(".paf");
        let temp_file = if let Some(ref temp_dir) = self.config.temp_dir {
            builder.tempfile_in(temp_dir)?
        } else {
            builder.tempfile()?
        };
        Ok(temp_file)
    }

    /// Run a wfmash command and capture stdout as PAF bytes.
    fn run_command(&self, mut cmd: std::process::Command) -> Result<Vec<u8>> {
        self.log_command(&cmd);

        let output = cmd
            .output()
            .map_err(|e| WfmashError::ExecutionFailed(format!("Failed to run wfmash: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WfmashError::ExecutionFailed(format!(
                "wfmash exited with code {:?}\nstderr: {}",
                output.status.code(),
                stderr
            )));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }

        eprintln!(
            "[wfmash] Completed, output size: {} bytes",
            output.stdout.len()
        );
        Ok(output.stdout)
    }

    /// Run a wfmash command and redirect stdout to a file.
    fn run_command_to_file(
        &self,
        mut cmd: std::process::Command,
        temp_file: NamedTempFile,
    ) -> Result<NamedTempFile> {
        self.log_command(&cmd);

        let output_file = std::fs::File::create(temp_file.path())?;
        cmd.stdout(output_file);

        let status = cmd
            .status()
            .map_err(|e| WfmashError::ExecutionFailed(format!("Failed to run wfmash: {e}")))?;

        if !status.success() {
            return Err(WfmashError::ExecutionFailed(format!(
                "wfmash exited with code {:?}",
                status.code(),
            )));
        }

        let file_size = std::fs::metadata(temp_file.path())?.len();
        eprintln!(
            "[wfmash] Completed, output file: {} ({} bytes)",
            temp_file.path().display(),
            file_size
        );
        Ok(temp_file)
    }

    /// Log the command being executed.
    fn log_command(&self, cmd: &std::process::Command) {
        let program = cmd.get_program().to_string_lossy();
        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        eprintln!("[wfmash] Running: {} {}", program, args.join(" "));
    }
}
