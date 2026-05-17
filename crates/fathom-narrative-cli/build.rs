//! Build script for fathom-narrative-cli.
//!
//! When built with `--features=ferrox-mip`, the binary links against
//! HiGHS (via `converge-ferrox-highs-sys`). The HiGHS sys crate sets an
//! rpath on its own crate's link line, but those flags don't propagate
//! to *binary* targets in this workspace — so a fresh build runs into a
//! macOS dyld error looking for `libhighs.1.dylib` in the standard
//! search paths.
//!
//! This script reads `FERROX_HIGHS_ROOT` (set by `.cargo/config.toml` to
//! the standard local checkout layout) and emits a `-Wl,-rpath,…` flag
//! for *this crate's binaries*, so `cargo run` and integration tests
//! work without a `DYLD_LIBRARY_PATH` wrapper.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FERROX_HIGHS_ROOT");
    println!("cargo:rerun-if-env-changed=FERROX_ORTOOLS_ROOT");

    // `rustc-link-arg-bins` applies only to bin targets — the right scope,
    // since the libhighs dylib search is a runtime concern of the executable,
    // not of the (workspace) library crates.
    if std::env::var_os("CARGO_FEATURE_FERROX_MIP").is_some()
        && let Ok(highs_root) = std::env::var("FERROX_HIGHS_ROOT")
    {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,{highs_root}/lib");
    }
    // Ferrox CP-SAT / OR-Tools wiring slot when that feature ships.
    // if std::env::var_os("CARGO_FEATURE_FERROX_CP").is_some() { ... }
}
