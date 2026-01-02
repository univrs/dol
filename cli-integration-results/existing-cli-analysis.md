# Existing CLI Analysis

## File Structure Patterns

All 8 CLI binaries in `src/bin/` follow consistent patterns:

| Binary | Purpose | Lines |
|--------|---------|-------|
| dol-parse.rs | Parse and validate files | 430 |
| dol-check.rs | Validation and coverage checking | 610 |
| dol-codegen.rs | Code generation | 266 |
| dol-test.rs | Test generation from specs | 562 |
| dol-migrate.rs | Version migration tool | 297 |
| dol-build-crate.rs | Generate complete Rust crates | 171 |
| dol-mcp.rs | Model Context Protocol server | 228 |
| wasm-stress-test.rs | WASM pipeline testing | 268 |

## Feature Gating

All binaries marked with `required-features = ["cli"]` in Cargo.toml:
- CLI feature gates: `clap`, `anyhow`, `colored`, `regex`, `serde`
- Additional features: `wasm`, `mlir` (for specialized binaries)

## Clap Derive Patterns

```rust
#[derive(Parser, Debug)]
#[command(name = "dol-<command>")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long, value_enum, default_value = "pretty")]
    format: OutputFormat,
}
```

## Error Handling Pattern

```rust
fn main() -> ExitCode {
    // Use Result<T, String> pattern
    // Colored output with colored crate
    if failed > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
```

## Key Patterns for vudo CLI

1. Use clap subcommands (like dol-test)
2. Support --quiet and --verbose flags
3. Multiple output formats (pretty, json)
4. Colored terminal output
5. Return proper exit codes
