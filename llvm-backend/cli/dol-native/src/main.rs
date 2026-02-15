use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use inkwell::context::Context as LlvmContext;

use dol_codegen_llvm::hir_lowering::HirLowering;
use dol_codegen_llvm::targets::Target;
use dol_codegen_llvm::LlvmCodegen;

#[derive(Parser)]
#[command(name = "dol-native")]
#[command(about = "DOL native compiler — compile DOL to machine code via LLVM")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a DOL file to a native object file
    Build {
        /// Input DOL source file
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target architecture
        #[arg(short, long, default_value = "x86_64-unknown-linux-gnu")]
        target: String,
    },

    /// Emit LLVM IR for a DOL file (for debugging)
    EmitIr {
        /// Input DOL source file
        input: PathBuf,
    },

    /// List supported compilation targets
    Targets,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input,
            output,
            target,
        } => cmd_build(&input, output, &target),
        Commands::EmitIr { input } => cmd_emit_ir(&input),
        Commands::Targets => cmd_targets(),
    }
}

/// Compile a DOL file to a native object file.
fn cmd_build(input: &PathBuf, output: Option<PathBuf>, target_str: &str) -> Result<()> {
    let target: Target = target_str.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    let source = std::fs::read_to_string(input)
        .with_context(|| format!("failed to read {}", input.display()))?;

    // Parse and lower to HIR
    let (hir, ctx) =
        metadol::lower::lower_file(&source).map_err(|e| anyhow::anyhow!("parse error: {}", e))?;

    // Create LLVM codegen
    let llvm_context = LlvmContext::create();
    let stem = input.file_stem().unwrap().to_string_lossy();
    let codegen = LlvmCodegen::new(&llvm_context, &stem, target.triple())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Lower HIR to LLVM IR
    {
        let mut lowering = HirLowering::new(codegen.context(), codegen.module(), &ctx.symbols);
        lowering
            .lower_module(&hir)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    // Determine output path
    let out_path = output.unwrap_or_else(|| {
        let ext = target.object_extension();
        input.with_extension(ext)
    });

    // Emit object file
    codegen
        .emit_object(&out_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    eprintln!(
        "compiled {} -> {} ({})",
        input.display(),
        out_path.display(),
        target.display_name()
    );
    Ok(())
}

/// Emit LLVM IR for a DOL file.
fn cmd_emit_ir(input: &PathBuf) -> Result<()> {
    let source = std::fs::read_to_string(input)
        .with_context(|| format!("failed to read {}", input.display()))?;

    // Parse and lower to HIR
    let (hir, ctx) =
        metadol::lower::lower_file(&source).map_err(|e| anyhow::anyhow!("parse error: {}", e))?;

    // Create LLVM codegen (use host target)
    let llvm_context = LlvmContext::create();
    let stem = input.file_stem().unwrap().to_string_lossy();
    let codegen = LlvmCodegen::new(&llvm_context, &stem, "x86_64-unknown-linux-gnu")
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Lower HIR to LLVM IR
    {
        let mut lowering = HirLowering::new(codegen.context(), codegen.module(), &ctx.symbols);
        lowering
            .lower_module(&hir)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    // Print LLVM IR
    println!("{}", codegen.emit_ir());
    Ok(())
}

/// List supported targets.
fn cmd_targets() -> Result<()> {
    println!("Supported targets:");
    println!();
    for target in Target::all() {
        println!("  {:40} {}", target.triple(), target.display_name());
    }
    Ok(())
}
