//! Target Architecture Definitions
//!
//! Defines supported compilation targets and their configurations.

use std::fmt;
use std::str::FromStr;

/// Supported compilation targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// ARM64 Linux (aarch64-unknown-linux-gnu)
    Aarch64Linux,
    /// ARM64 macOS / Apple Silicon (aarch64-apple-darwin)
    Aarch64Darwin,
    /// RISC-V 64-bit Linux (riscv64gc-unknown-linux-gnu)
    Riscv64Linux,
    /// x86-64 Linux (x86_64-unknown-linux-gnu)
    X86_64Linux,
    /// x86-64 Windows (x86_64-pc-windows-msvc)
    X86_64Windows,
}

impl Target {
    /// Get the LLVM target triple string
    pub fn triple(&self) -> &'static str {
        match self {
            Target::Aarch64Linux => "aarch64-unknown-linux-gnu",
            Target::Aarch64Darwin => "aarch64-apple-darwin",
            Target::Riscv64Linux => "riscv64gc-unknown-linux-gnu",
            Target::X86_64Linux => "x86_64-unknown-linux-gnu",
            Target::X86_64Windows => "x86_64-pc-windows-msvc",
        }
    }

    /// Get the CPU features string
    pub fn features(&self) -> &'static str {
        match self {
            Target::Aarch64Linux => "+neon",
            Target::Aarch64Darwin => "+neon,+fp-armv8",
            Target::Riscv64Linux => "+m,+a,+f,+d,+c",
            Target::X86_64Linux => "+sse2",
            Target::X86_64Windows => "+sse2",
        }
    }

    /// Get the default CPU model
    pub fn cpu(&self) -> &'static str {
        match self {
            Target::Aarch64Linux => "generic",
            Target::Aarch64Darwin => "apple-m1",
            Target::Riscv64Linux => "generic-rv64",
            Target::X86_64Linux => "x86-64",
            Target::X86_64Windows => "x86-64",
        }
    }

    /// Get the data layout string for LLVM
    pub fn data_layout(&self) -> &'static str {
        match self {
            Target::Aarch64Linux | Target::Aarch64Darwin => {
                "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
            }
            Target::Riscv64Linux => "e-m:e-p:64:64-i64:64-i128:128-n64-S128",
            Target::X86_64Linux => {
                "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            }
            Target::X86_64Windows => {
                "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            }
        }
    }

    /// Get the object file extension for this target
    pub fn object_extension(&self) -> &'static str {
        match self {
            Target::X86_64Windows => "obj",
            _ => "o",
        }
    }

    /// Get the static library extension for this target
    pub fn static_lib_extension(&self) -> &'static str {
        match self {
            Target::X86_64Windows => "lib",
            _ => "a",
        }
    }

    /// Get the executable extension for this target
    pub fn executable_extension(&self) -> &'static str {
        match self {
            Target::X86_64Windows => "exe",
            _ => "",
        }
    }

    /// List all supported targets
    pub fn all() -> &'static [Target] {
        &[
            Target::Aarch64Linux,
            Target::Aarch64Darwin,
            Target::Riscv64Linux,
            Target::X86_64Linux,
            Target::X86_64Windows,
        ]
    }

    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Target::Aarch64Linux => "ARM64 Linux",
            Target::Aarch64Darwin => "ARM64 macOS (Apple Silicon)",
            Target::Riscv64Linux => "RISC-V 64-bit Linux",
            Target::X86_64Linux => "x86-64 Linux",
            Target::X86_64Windows => "x86-64 Windows",
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.triple())
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aarch64-unknown-linux-gnu" | "aarch64-linux" | "arm64-linux" => {
                Ok(Target::Aarch64Linux)
            }
            "aarch64-apple-darwin" | "aarch64-darwin" | "arm64-darwin" | "arm64-macos" => {
                Ok(Target::Aarch64Darwin)
            }
            "riscv64gc-unknown-linux-gnu" | "riscv64-linux" | "riscv64" => Ok(Target::Riscv64Linux),
            "x86_64-unknown-linux-gnu" | "x86_64-linux" | "x64-linux" => Ok(Target::X86_64Linux),
            "x86_64-pc-windows-msvc" | "x86_64-windows" | "x64-windows" => {
                Ok(Target::X86_64Windows)
            }
            _ => Err(format!("unsupported target: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_parsing() {
        assert_eq!("aarch64-linux".parse::<Target>(), Ok(Target::Aarch64Linux));
        assert_eq!("arm64-macos".parse::<Target>(), Ok(Target::Aarch64Darwin));
        assert_eq!("riscv64".parse::<Target>(), Ok(Target::Riscv64Linux));
        assert_eq!("x64-linux".parse::<Target>(), Ok(Target::X86_64Linux));
        assert!("unknown".parse::<Target>().is_err());
    }

    #[test]
    fn test_all_targets() {
        assert_eq!(Target::all().len(), 5);
    }
}
