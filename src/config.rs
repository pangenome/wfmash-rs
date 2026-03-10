//! Configuration options for wfmash alignment operations.
//!
//! This module provides a builder pattern for configuring alignment parameters,
//! mapping to wfmash CLI flags.

use std::path::PathBuf;

/// Configuration for wfmash alignment operations.
///
/// Use the builder pattern to construct configurations with non-default values.
///
/// # Example
/// ```
/// use wfmash_rs::Config;
///
/// let config = Config::builder()
///     .num_threads(8)
///     .segment_length(5000)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of threads (-t)
    pub num_threads: usize,

    /// Percent identity for mapping/alignment (-p).
    /// Accepts float or ANI preset string like "ani50-2".
    pub map_pct_identity: Option<String>,

    /// Number of secondary mappings to retain, plane sweep (-n)
    pub num_mappings: Option<usize>,

    /// Minimum block length to filter (-l)
    pub block_length: Option<u64>,

    /// Kmer size (-k)
    pub kmer_size: Option<usize>,

    /// Window size for sketching (-w)
    pub window_size: Option<String>,

    /// Segment length (-s)
    pub segment_length: Option<usize>,

    /// Include self mappings (-X)
    pub self_mappings: bool,

    /// PanSN prefix delimiter for grouping (-Y)
    pub prefix_delimiter: Option<char>,

    /// Only compute lower-triangular mappings (-L)
    pub lower_triangular: bool,

    /// Mapping only, no base-level alignment (-m)
    pub mapping_only: bool,

    /// Target prefix for filtering (-T)
    pub target_prefix: Option<String>,

    /// Query prefix list for filtering (-Q)
    pub query_prefixes: Option<String>,

    /// Disable mapping filtering (-f)
    pub no_filter: bool,

    /// Enable one-to-one mapping (-4)
    pub one_to_one: bool,

    /// Index batch size (-b)
    pub index_batch_size: Option<String>,

    /// Chain jump / gap (-c)
    pub chain_jump: Option<String>,

    /// Max mapping length (-P)
    pub max_length: Option<String>,

    /// Temporary directory (sets TMPDIR env var)
    pub temp_dir: Option<PathBuf>,

    /// Extra arguments to pass directly to wfmash
    pub extra_args: Vec<String>,

    /// File containing allowed query<TAB>target pairs (--pairs-file)
    pub pairs_file: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            num_threads: num_cpus::get().max(1),
            map_pct_identity: None,
            num_mappings: None,
            block_length: None,
            kmer_size: None,
            window_size: None,
            segment_length: None,
            self_mappings: false,
            prefix_delimiter: None,
            lower_triangular: false,
            mapping_only: false,
            target_prefix: None,
            query_prefixes: None,
            no_filter: false,
            one_to_one: false,
            index_batch_size: None,
            chain_jump: None,
            max_length: None,
            temp_dir: None,
            extra_args: Vec::new(),
            pairs_file: None,
        }
    }
}

impl Config {
    /// Creates a new configuration builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Build the wfmash CLI arguments from this config.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.push(format!("-t{}", self.num_threads));

        if let Some(ref pct) = self.map_pct_identity {
            args.push(format!("-p{}", pct));
        }

        if let Some(n) = self.num_mappings {
            args.push(format!("-n{}", n));
        }

        if let Some(l) = self.block_length {
            args.push(format!("-l{}", l));
        }

        if let Some(k) = self.kmer_size {
            args.push(format!("-k{}", k));
        }

        if let Some(ref w) = self.window_size {
            args.push(format!("-w{}", w));
        }

        if let Some(s) = self.segment_length {
            args.push(format!("-s{}", s));
        }

        if self.self_mappings {
            args.push("-X".to_string());
        }

        if let Some(d) = self.prefix_delimiter {
            args.push(format!("-Y{}", d));
        }

        if self.lower_triangular {
            args.push("-L".to_string());
        }

        if self.mapping_only {
            args.push("-m".to_string());
        }

        if let Some(ref t) = self.target_prefix {
            args.push(format!("-T{}", t));
        }

        if let Some(ref q) = self.query_prefixes {
            args.push(format!("-Q{}", q));
        }

        if self.no_filter {
            args.push("-f".to_string());
        }

        if self.one_to_one {
            args.push("-4".to_string());
        }

        if let Some(ref b) = self.index_batch_size {
            args.push(format!("-b{}", b));
        }

        if let Some(ref c) = self.chain_jump {
            args.push(format!("-c{}", c));
        }

        if let Some(ref p) = self.max_length {
            args.push(format!("-P{}", p));
        }

        if let Some(ref pf) = self.pairs_file {
            args.push("--pairs-file".to_string());
            args.push(pf.to_string_lossy().to_string());
        }

        args.extend(self.extra_args.iter().cloned());

        args
    }
}

/// Builder for constructing Config instances.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Sets the number of threads (-t).
    pub fn num_threads(mut self, threads: usize) -> Self {
        assert!(threads > 0, "Number of threads must be positive");
        self.config.num_threads = threads;
        self
    }

    /// Sets the percent identity threshold for mapping/alignment (-p).
    /// Accepts a float value (e.g., 90.0) or an ANI preset string (e.g., "ani50-2").
    pub fn map_pct_identity(mut self, pct: &str) -> Self {
        self.config.map_pct_identity = Some(pct.to_string());
        self
    }

    /// Sets the number of secondary mappings to retain (-n).
    pub fn num_mappings(mut self, n: usize) -> Self {
        self.config.num_mappings = Some(n);
        self
    }

    /// Sets the minimum block length filter (-l).
    pub fn block_length(mut self, length: u64) -> Self {
        self.config.block_length = Some(length);
        self
    }

    /// Sets the kmer size (-k).
    pub fn kmer_size(mut self, k: usize) -> Self {
        self.config.kmer_size = Some(k);
        self
    }

    /// Sets the window size for sketching (-w). Accepts suffixed values like "1k".
    pub fn window_size(mut self, w: &str) -> Self {
        self.config.window_size = Some(w.to_string());
        self
    }

    /// Sets the segment length (-s).
    pub fn segment_length(mut self, s: usize) -> Self {
        self.config.segment_length = Some(s);
        self
    }

    /// Enable self-mapping mode (-X).
    pub fn self_mappings(mut self, enabled: bool) -> Self {
        self.config.self_mappings = enabled;
        self
    }

    /// Sets the PanSN prefix delimiter (-Y).
    pub fn prefix_delimiter(mut self, d: char) -> Self {
        self.config.prefix_delimiter = Some(d);
        self
    }

    /// Enable lower-triangular only mode (-L).
    pub fn lower_triangular(mut self, enabled: bool) -> Self {
        self.config.lower_triangular = enabled;
        self
    }

    /// Enable mapping-only mode, no base-level alignment (-m).
    pub fn mapping_only(mut self, enabled: bool) -> Self {
        self.config.mapping_only = enabled;
        self
    }

    /// Sets the target prefix filter (-T).
    pub fn target_prefix(mut self, prefix: &str) -> Self {
        self.config.target_prefix = Some(prefix.to_string());
        self
    }

    /// Sets the query prefix list (-Q).
    pub fn query_prefixes(mut self, prefixes: &str) -> Self {
        self.config.query_prefixes = Some(prefixes.to_string());
        self
    }

    /// Disable mapping filtering (-f).
    pub fn no_filter(mut self, enabled: bool) -> Self {
        self.config.no_filter = enabled;
        self
    }

    /// Enable one-to-one mapping (-4).
    pub fn one_to_one(mut self, enabled: bool) -> Self {
        self.config.one_to_one = enabled;
        self
    }

    /// Sets the index batch size (-b). Accepts suffixed values like "10M".
    pub fn index_batch_size(mut self, b: &str) -> Self {
        self.config.index_batch_size = Some(b.to_string());
        self
    }

    /// Sets the chain jump / gap (-c). Accepts suffixed values like "2k".
    pub fn chain_jump(mut self, c: &str) -> Self {
        self.config.chain_jump = Some(c.to_string());
        self
    }

    /// Sets the max mapping length (-P). Accepts suffixed values like "50k".
    pub fn max_length(mut self, p: &str) -> Self {
        self.config.max_length = Some(p.to_string());
        self
    }

    /// Sets the temporary directory.
    pub fn temp_dir(mut self, path: PathBuf) -> Self {
        self.config.temp_dir = Some(path);
        self
    }

    /// Add extra arguments to pass directly to wfmash.
    pub fn extra_args(mut self, args: Vec<String>) -> Self {
        self.config.extra_args = args;
        self
    }

    /// Sets the pairs file for allowed query-target pairs (--pairs-file).
    pub fn pairs_file(mut self, path: PathBuf) -> Self {
        self.config.pairs_file = Some(path);
        self
    }

    /// Builds the final Config instance.
    pub fn build(self) -> Config {
        self.config
    }
}
