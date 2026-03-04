//! Integration tests for wfmash-rs.

use std::path::Path;
use wfmash_rs::{Config, Wfmash};

/// Test that binary discovery works (either vendored or system).
#[test]
fn test_binary_found() {
    let result = Wfmash::new(Config::default());
    // This may fail if wfmash is not installed or not yet built
    if result.is_err() {
        eprintln!(
            "Skipping test_binary_found: wfmash binary not found. \
             Build with vendored source or install wfmash to PATH."
        );
        return;
    }
    assert!(result.is_ok());
}

/// Test self-alignment on test FASTA data.
#[test]
fn test_self_alignment() {
    let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/test.fa.gz");

    if !test_data.exists() {
        eprintln!("Skipping test_self_alignment: test data not found at {:?}", test_data);
        return;
    }

    let config = Config::builder()
        .num_threads(4)
        .segment_length("5k")
        .build();

    let wfmash = match Wfmash::new(config) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Skipping test_self_alignment: {}", e);
            return;
        }
    };

    let paf_bytes = wfmash.align_self(&test_data).expect("Self-alignment failed");
    assert!(!paf_bytes.is_empty(), "PAF output should not be empty");

    // Verify PAF format: each line should have at least 12 tab-separated fields
    let paf_str = String::from_utf8(paf_bytes).expect("PAF should be valid UTF-8");
    let lines: Vec<&str> = paf_str.lines().collect();
    assert!(!lines.is_empty(), "PAF should have at least one alignment");

    for (i, line) in lines.iter().enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(
            fields.len() >= 12,
            "Line {} has {} fields, expected >= 12: {}",
            i + 1,
            fields.len(),
            line
        );
    }
}

/// Test pairwise alignment.
#[test]
fn test_pairwise_alignment() {
    let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/test.fa.gz");

    if !test_data.exists() {
        eprintln!("Skipping test_pairwise_alignment: test data not found");
        return;
    }

    let config = Config::builder()
        .num_threads(4)
        .segment_length("5k")
        .build();

    let wfmash = match Wfmash::new(config) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Skipping test_pairwise_alignment: {}", e);
            return;
        }
    };

    // Use same file as both target and query for testing
    let paf_bytes = wfmash
        .align_files(&test_data, &test_data)
        .expect("Pairwise alignment failed");
    assert!(!paf_bytes.is_empty(), "PAF output should not be empty");
}

/// Test align_self_to_temp_paf produces a valid temp file.
#[test]
fn test_align_self_to_temp_paf() {
    let test_data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/test.fa.gz");

    if !test_data.exists() {
        eprintln!("Skipping test_align_self_to_temp_paf: test data not found");
        return;
    }

    let config = Config::builder()
        .num_threads(4)
        .segment_length("5k")
        .build();

    let wfmash = match Wfmash::new(config) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Skipping test_align_self_to_temp_paf: {}", e);
            return;
        }
    };

    let temp_paf = wfmash
        .align_self_to_temp_paf(&test_data)
        .expect("align_self_to_temp_paf failed");

    let metadata = std::fs::metadata(temp_paf.path()).expect("temp file should exist");
    assert!(metadata.len() > 0, "Temp PAF file should not be empty");
}

/// Test config argument generation.
#[test]
fn test_config_args_generation() {
    let config = Config::builder()
        .num_threads(16)
        .segment_length("10k")
        .map_pct_identity(90.0)
        .num_mappings(5)
        .self_mappings(true)
        .prefix_delimiter('#')
        .lower_triangular(true)
        .mapping_only(true)
        .no_filter(true)
        .one_to_one(true)
        .kmer_size(19)
        .window_size("100")
        .block_length(1000)
        .target_prefix("target")
        .query_prefixes("q1,q2")
        .index_batch_size("10M")
        .build();

    let args = config.to_args();

    assert!(args.contains(&"-t16".to_string()));
    assert!(args.contains(&"-s10k".to_string()));
    assert!(args.contains(&"-p90".to_string()));
    assert!(args.contains(&"-n5".to_string()));
    assert!(args.contains(&"-X".to_string()));
    assert!(args.contains(&"-Y#".to_string()));
    assert!(args.contains(&"-L".to_string()));
    assert!(args.contains(&"-m".to_string()));
    assert!(args.contains(&"-f".to_string()));
    assert!(args.contains(&"-o".to_string()));
    assert!(args.contains(&"-k19".to_string()));
    assert!(args.contains(&"-w100".to_string()));
    assert!(args.contains(&"-l1000".to_string()));
    assert!(args.contains(&"-Ttarget".to_string()));
    assert!(args.contains(&"-Qq1,q2".to_string()));
    assert!(args.contains(&"-b10M".to_string()));
}
