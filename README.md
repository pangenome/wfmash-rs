# wfmash-rs

[![CI](https://github.com/pangenome/wfmash-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/pangenome/wfmash-rs/actions/workflows/ci.yml)

Rust bindings for the [wfmash](https://github.com/waveygang/wfmash) whole-genome aligner.

## Features

- All-vs-all whole-genome alignment (self-alignment)
- Pairwise genome alignment (query vs target)
- Configurable parameters via builder pattern
- Temporary file output with automatic cleanup
- Vendored wfmash binary (built from source during `cargo build`)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
wfmash-rs = { git = "https://github.com/pangenome/wfmash-rs" }
```

### Example

```rust
use wfmash_rs::{Wfmash, Config};
use std::path::Path;

let config = Config::builder()
    .num_threads(8)
    .segment_length(5000)
    .build();

let wfmash = Wfmash::new(config)?;

// Self-alignment (all-vs-all)
let paf_bytes = wfmash.align_self(Path::new("genomes.fa.gz"))?;

// Pairwise alignment (target, query)
let paf_bytes = wfmash.align_files(
    Path::new("target.fa"),
    Path::new("query.fa"),
)?;

// Write to temporary file (auto-deleted when dropped)
let temp_paf = wfmash.align_self_to_temp_paf(Path::new("genomes.fa.gz"))?;
println!("PAF at: {}", temp_paf.path().display());
```

### Configuration

```rust
let config = Config::builder()
    .num_threads(8)              // -t: threads (default: all CPUs)
    .segment_length(5000)        // -s: segment length
    .block_length(10000)         // -l: minimum alignment block length
    .map_pct_identity("90")      // -p: minimum mapping identity %
    .num_mappings(5)             // -n: max mappings per segment
    .kmer_size(19)               // -k: kmer size
    .self_mappings(true)         // -X: include self mappings
    .prefix_delimiter('#')       // -Y: PanSN prefix delimiter
    .lower_triangular(true)      // -L: lower-triangular output only
    .one_to_one(true)            // -4: one-to-one mapping
    .no_filter(true)             // -f: disable mapping filtering
    .pairs_file(PathBuf::from("pairs.tsv"))  // --pairs-file: restrict to allowed pairs
    .temp_dir(PathBuf::from("/tmp"))  // TMPDIR for temp files
    .build();
```

## Build requirements

- C++ compiler with C++17 support
- CMake >= 3.5
- GSL (GNU Scientific Library)
- htslib
- jemalloc
- zlib

On Ubuntu/Debian:
```bash
sudo apt-get install cmake libgsl-dev libhts-dev libjemalloc-dev zlib1g-dev
```

On macOS:
```bash
brew install cmake gsl htslib jemalloc
```

## License

MIT
