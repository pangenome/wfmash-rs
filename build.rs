/// Build script for compiling wfmash from vendored source via CMake.
///
/// This compiles wfmash from deps/wfmash/ and places the binary in $OUT_DIR.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=deps/wfmash");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wfmash_src = manifest_dir.join("deps").join("wfmash");

    // Check if vendored source exists
    if !wfmash_src.join("CMakeLists.txt").exists() {
        println!(
            "cargo:warning=Vendored wfmash source not found at {}. \
             Skipping build — will rely on system wfmash binary.",
            wfmash_src.display()
        );
        return;
    }

    let wfmash_bin = out_dir.join("wfmash");

    // Skip rebuild if binary already exists
    if wfmash_bin.exists() {
        println!("cargo:warning=wfmash binary already built, skipping rebuild");
        println!("cargo:rustc-env=WFMASH_BIN_DIR={}", out_dir.display());
        return;
    }

    println!("cargo:warning=Building wfmash from vendored source...");

    let build_dir = out_dir.join("wfmash-build");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Initialize git submodules if needed (WFA2-lib, etc.)
    init_submodules(&wfmash_src);

    // CMake configure
    let mut cmake_args = vec![
        wfmash_src.to_str().unwrap().to_string(),
        "-DCMAKE_BUILD_TYPE=Release".to_string(),
        "-DVENDOR_EVERYTHING=ON".to_string(),
    ];

    // On macOS, Apple clang doesn't support -fopenmp directly.
    // We need to help CMake find libomp (typically installed via `brew install libomp`).
    #[cfg(target_os = "macos")]
    {
        // Try to find libomp via brew
        if let Ok(output) = Command::new("brew").args(["--prefix", "libomp"]).output() {
            if output.status.success() {
                let libomp_prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("cargo:warning=Found libomp at: {}", libomp_prefix);
                cmake_args.push(format!(
                    "-DOpenMP_C_FLAGS=-Xpreprocessor -fopenmp -I{}/include",
                    libomp_prefix
                ));
                cmake_args.push(format!(
                    "-DOpenMP_CXX_FLAGS=-Xpreprocessor -fopenmp -I{}/include",
                    libomp_prefix
                ));
                cmake_args.push(format!("-DOpenMP_C_LIB_NAMES=omp",));
                cmake_args.push(format!("-DOpenMP_CXX_LIB_NAMES=omp",));
                cmake_args.push(format!(
                    "-DOpenMP_omp_LIBRARY={}/lib/libomp.dylib",
                    libomp_prefix
                ));
            }
        }
    }

    let cmake_arg_refs: Vec<&str> = cmake_args.iter().map(|s| s.as_str()).collect();
    let configure_status = Command::new("cmake")
        .args(&cmake_arg_refs)
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run cmake configure. Is cmake installed?");

    if !configure_status.success() {
        panic!("cmake configure failed");
    }

    // CMake build
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("cargo:warning=Building wfmash with {} threads...", num_cpus);

    let build_status = Command::new("cmake")
        .args(["--build", ".", "--", &format!("-j{}", num_cpus)])
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run cmake build");

    if !build_status.success() {
        panic!("cmake build failed");
    }

    // Copy binary to OUT_DIR
    let built_binary = build_dir.join("bin").join("wfmash");
    if !built_binary.exists() {
        panic!(
            "wfmash binary not found at expected path: {}",
            built_binary.display()
        );
    }

    std::fs::copy(&built_binary, &wfmash_bin).expect("Failed to copy wfmash binary to OUT_DIR");

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&wfmash_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&wfmash_bin, perms).unwrap();
    }

    println!("cargo:warning=wfmash binary built successfully");
    println!("cargo:rustc-env=WFMASH_BIN_DIR={}", out_dir.display());
}

/// Initialize git submodules in the wfmash source tree if they're not already present.
fn init_submodules(wfmash_src: &std::path::Path) {
    // Check if WFA2-lib submodule is populated
    let wfa2_dir = wfmash_src.join("deps").join("WFA2-lib");
    if wfa2_dir.exists() && wfa2_dir.join("CMakeLists.txt").exists() {
        return; // Submodule already initialized
    }

    println!("cargo:warning=Initializing wfmash git submodules...");

    let status = Command::new("git")
        .args(["submodule", "update", "--init", "--recursive"])
        .current_dir(wfmash_src)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Git submodules initialized successfully");
        }
        Ok(s) => {
            println!(
                "cargo:warning=Git submodule init returned non-zero: {:?}",
                s.code()
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run git submodule update: {}. \
                 Submodules may need to be initialized manually.",
                e
            );
        }
    }
}
