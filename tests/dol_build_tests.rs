//! Integration tests for dol-build CLI command.
//!
//! These tests verify the end-to-end build pipeline:
//! 1. Manifest parsing (Spirit.dol)
//! 2. DOL → Rust code generation
//! 3. Rust → WASM compilation
//! 4. WASM → JS bindings
//! 5. Manifest.json generation

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Path to the dol-build binary (once implemented).
fn dol_build_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("dol-build");
    path
}

/// Helper to run dol-build command.
fn run_dol_build(working_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(dol_build_binary())
        .current_dir(working_dir)
        .args(args)
        .output()
        .expect("Failed to execute dol-build")
}

/// Verify that the binary exists and is executable.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_dol_build_binary_exists() {
    let binary = dol_build_binary();
    assert!(
        binary.exists(),
        "dol-build binary not found at {}. Run: cargo build --release --features cli --bin dol-build",
        binary.display()
    );
}

/// Test building the game-of-life example.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_build_game_of_life() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_of_life_dir = manifest_dir.join("examples/spirits/game-of-life");

    // Run dol-build
    let output = run_dol_build(&game_of_life_dir, &["--verbose"]);

    // Should succeed
    assert!(
        output.status.success(),
        "dol-build failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Check expected outputs
    let target_dir = game_of_life_dir.join("target/spirit");

    assert!(
        target_dir.join("game_of_life.wasm").exists(),
        "WASM output not found: {}",
        target_dir.join("game_of_life.wasm").display()
    );

    assert!(
        target_dir.join("game_of_life.js").exists(),
        "JS bindings not found: {}",
        target_dir.join("game_of_life.js").display()
    );

    assert!(
        target_dir.join("manifest.json").exists(),
        "Manifest not found: {}",
        target_dir.join("manifest.json").display()
    );

    // Verify manifest.json content
    let manifest_content =
        fs::read_to_string(target_dir.join("manifest.json")).expect("Failed to read manifest.json");

    assert!(
        manifest_content.contains("GameOfLife") || manifest_content.contains("game-of-life"),
        "Manifest should contain spirit name"
    );
    assert!(
        manifest_content.contains("0.1.0"),
        "Manifest should contain version"
    );
}

/// Test building with verbose output.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_build_verbose_output() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_of_life_dir = manifest_dir.join("examples/spirits/game-of-life");

    let output = run_dol_build(&game_of_life_dir, &["--verbose"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should show build steps
    assert!(
        combined.contains("DOL") || combined.contains("dol"),
        "Should mention DOL compilation"
    );
    assert!(
        combined.contains("Rust") || combined.contains("rust"),
        "Should mention Rust compilation"
    );
    assert!(
        combined.contains("WASM") || combined.contains("wasm"),
        "Should mention WASM compilation"
    );
}

/// Test that dol-build fails gracefully when Spirit.dol is missing.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_build_missing_manifest() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_dol_build(temp_dir.path(), &[]);

    // Should fail
    assert!(
        !output.status.success(),
        "dol-build should fail when Spirit.dol is missing"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Spirit.dol") || stderr.contains("manifest"),
        "Error message should mention missing manifest"
    );
}

/// Test building with a custom output directory.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_build_custom_output_dir() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_of_life_dir = manifest_dir.join("examples/spirits/game-of-life");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let custom_output = temp_dir.path().join("custom-build");

    let output = run_dol_build(
        &game_of_life_dir,
        &["--output", custom_output.to_str().unwrap()],
    );

    assert!(
        output.status.success(),
        "dol-build with custom output should succeed"
    );

    assert!(
        custom_output.join("game_of_life.wasm").exists(),
        "WASM should be in custom output directory"
    );
}

/// Test that dol-build is faster than build.sh for the same project.
#[test]
#[ignore = "Requires dol-build implementation and performance testing"]
fn test_build_performance_vs_shell_script() {
    use std::time::Instant;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_of_life_dir = manifest_dir.join("examples/spirits/game-of-life");

    // Time build.sh
    let start = Instant::now();
    let shell_output = Command::new("bash")
        .arg("build.sh")
        .current_dir(&game_of_life_dir)
        .output()
        .expect("Failed to run build.sh");
    let shell_duration = start.elapsed();

    assert!(
        shell_output.status.success(),
        "build.sh should succeed for comparison"
    );

    // Clean build artifacts
    let _ = fs::remove_dir_all(game_of_life_dir.join("target"));
    let _ = fs::remove_dir_all(game_of_life_dir.join("codegen"));

    // Time dol-build
    let start = Instant::now();
    let build_output = run_dol_build(&game_of_life_dir, &[]);
    let build_duration = start.elapsed();

    assert!(
        build_output.status.success(),
        "dol-build should succeed for comparison"
    );

    eprintln!("build.sh: {:?}", shell_duration);
    eprintln!("dol-build: {:?}", build_duration);

    // dol-build should be comparable or faster
    // (Allow 20% overhead for now, can be tightened later)
    assert!(
        build_duration <= shell_duration * 12 / 10,
        "dol-build should be within 20% of build.sh performance"
    );
}

/// Test building multiple times (incremental builds).
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_incremental_build() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_of_life_dir = manifest_dir.join("examples/spirits/game-of-life");

    // First build
    let output1 = run_dol_build(&game_of_life_dir, &[]);
    assert!(output1.status.success(), "First build should succeed");

    // Get timestamp of WASM file
    let wasm_path = game_of_life_dir.join("target/spirit/game_of_life.wasm");
    let metadata1 = fs::metadata(&wasm_path).expect("WASM file should exist");
    let modified1 = metadata1.modified().expect("Should have modified time");

    // Second build (without changes)
    std::thread::sleep(std::time::Duration::from_secs(1));
    let output2 = run_dol_build(&game_of_life_dir, &[]);
    assert!(output2.status.success(), "Second build should succeed");

    // Check if incremental build skipped work
    let metadata2 = fs::metadata(&wasm_path).expect("WASM file should still exist");
    let modified2 = metadata2.modified().expect("Should have modified time");

    // For incremental builds, the timestamp should be the same
    // (or dol-build should report "up to date")
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    if modified1 == modified2 {
        // Good: file wasn't rebuilt
        eprintln!("Incremental build skipped WASM compilation (file unchanged)");
    } else if stderr2.contains("up to date") || stderr2.contains("cached") {
        // Good: dol-build detected no changes
        eprintln!("Incremental build detected by dol-build");
    } else {
        // Warning: full rebuild occurred
        eprintln!("WARNING: dol-build performed full rebuild. Consider caching optimization.");
    }
}

/// Test that --help flag works.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_help_flag() {
    let output = Command::new(dol_build_binary())
        .arg("--help")
        .output()
        .expect("Failed to run dol-build --help");

    assert!(output.status.success(), "dol-build --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dol-build") || stdout.contains("Build"),
        "Help should describe the command"
    );
}

/// Test that --version flag works.
#[test]
#[ignore = "Requires dol-build implementation"]
fn test_version_flag() {
    let output = Command::new(dol_build_binary())
        .arg("--version")
        .output()
        .expect("Failed to run dol-build --version");

    assert!(
        output.status.success(),
        "dol-build --version should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dol") && (stdout.contains("0.") || stdout.contains("1.")),
        "Version should show version number"
    );
}
