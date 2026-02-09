# Installing LLVM 17 for DOL LLVM Backend

The DOL LLVM Backend requires LLVM 17 to be installed on your system. This guide covers installation on various platforms.

---

## Ubuntu / Debian

### Option 1: APT Package Manager (Recommended)

```bash
# Add LLVM repository (if not already present)
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo add-apt-repository "deb http://apt.llvm.org/$(lsb_release -cs)/ llvm-toolchain-$(lsb_release -cs)-17 main"

# Update and install
sudo apt update
sudo apt install -y \
    llvm-17 \
    llvm-17-dev \
    llvm-17-tools \
    libllvm17 \
    clang-17 \
    lld-17

# Verify installation
llvm-config-17 --version  # Should output: 17.x.x

# Set environment variable (add to ~/.bashrc for persistence)
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
```

### Option 2: Build from Source

```bash
# Install build dependencies
sudo apt install -y \
    build-essential \
    cmake \
    ninja-build \
    python3 \
    git \
    zlib1g-dev \
    libxml2-dev

# Download LLVM 17
cd /tmp
wget https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/llvm-17.0.6.src.tar.xz
wget https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/cmake-17.0.6.src.tar.xz

tar xf llvm-17.0.6.src.tar.xz
tar xf cmake-17.0.6.src.tar.xz

cd llvm-17.0.6.src
mkdir build && cd build

# Configure (takes ~5 minutes)
cmake -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_ENABLE_RTTI=ON \
    -DLLVM_ENABLE_ZLIB=ON \
    -DCMAKE_INSTALL_PREFIX=/usr/local \
    ..

# Build (takes 30-60 minutes depending on CPU)
ninja -j$(nproc)

# Install
sudo ninja install

# Verify
llvm-config --version  # Should output: 17.0.6
```

---

## macOS

### Option 1: Homebrew (Recommended)

```bash
# Install LLVM 17
brew install llvm@17

# Set environment variable (add to ~/.zshrc or ~/.bash_profile)
export LLVM_SYS_170_PREFIX="$(brew --prefix llvm@17)"

# Verify
$(brew --prefix llvm@17)/bin/llvm-config --version
```

### Option 2: Build from Source

```bash
# Install Xcode command line tools
xcode-select --install

# Install CMake and Ninja
brew install cmake ninja

# Download and build (same as Linux instructions above)
# Use CMAKE_INSTALL_PREFIX=/usr/local/opt/llvm@17
```

---

## Windows (WSL2 Recommended)

### Option 1: WSL2 + Ubuntu (Recommended)

If using WSL2 with Ubuntu, follow the Ubuntu instructions above.

### Option 2: Native Windows (Advanced)

```powershell
# Download pre-built binaries
# Visit: https://github.com/llvm/llvm-project/releases/tag/llvmorg-17.0.6
# Download: LLVM-17.0.6-win64.exe

# Run installer and add to PATH
$env:LLVM_SYS_170_PREFIX = "C:\Program Files\LLVM"

# Verify
llvm-config --version
```

---

## Arch Linux

```bash
# Install from pacman
sudo pacman -S llvm17 clang17

# Verify
llvm-config-17 --version

# Set environment variable
export LLVM_SYS_170_PREFIX=/usr/lib/llvm17
```

---

## Fedora / RHEL / CentOS

```bash
# Install from dnf
sudo dnf install -y \
    llvm17 \
    llvm17-devel \
    clang17

# Verify
llvm-config-17 --version

# Set environment variable
export LLVM_SYS_170_PREFIX=/usr/lib64/llvm17
```

---

## Post-Installation Verification

After installing LLVM, verify the installation:

```bash
# Check LLVM version
llvm-config --version  # or llvm-config-17 --version

# Check environment variable
echo $LLVM_SYS_170_PREFIX

# Try building the project
cd ~/univrs-Sepah/llvm-backend
cargo clean
cargo build

# Expected output: Successful compilation of dol-codegen-llvm and vudo-runtime-native
```

---

## Troubleshooting

### Error: "No suitable version of LLVM was found"

**Solution:**
```bash
# Find LLVM installation
find /usr -name "llvm-config*" 2>/dev/null

# Set LLVM_SYS_170_PREFIX to the parent of bin/llvm-config
# Example: If llvm-config is at /usr/lib/llvm-17/bin/llvm-config
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
```

### Error: "LLVM version mismatch"

**Solution:**
Ensure you have exactly LLVM 17.x installed. The inkwell crate requires LLVM 17.

```bash
# Check version
llvm-config --version

# If wrong version, uninstall and reinstall LLVM 17
```

### Error: "libLLVM-17.so not found"

**Solution:**
```bash
# Find library
find /usr -name "libLLVM-17.so*" 2>/dev/null

# Add to LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/usr/lib/llvm-17/lib:$LD_LIBRARY_PATH

# Add to .bashrc for persistence
echo 'export LD_LIBRARY_PATH=/usr/lib/llvm-17/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
```

### Performance: Compilation is slow

**Tip:** Use `ninja` instead of `make` for faster parallel builds:
```bash
cmake -G Ninja ...
ninja -j$(nproc)
```

---

## Environment Variables Reference

Add these to `~/.bashrc`, `~/.zshrc`, or equivalent:

```bash
# LLVM 17 installation prefix
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17  # Adjust path as needed

# Add LLVM tools to PATH (optional but useful)
export PATH=$LLVM_SYS_170_PREFIX/bin:$PATH

# Add LLVM libraries to LD_LIBRARY_PATH (Linux)
export LD_LIBRARY_PATH=$LLVM_SYS_170_PREFIX/lib:$LD_LIBRARY_PATH

# Add LLVM libraries to DYLD_LIBRARY_PATH (macOS)
export DYLD_LIBRARY_PATH=$LLVM_SYS_170_PREFIX/lib:$DYLD_LIBRARY_PATH
```

Reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc on macOS
```

---

## Next Steps

After installing LLVM 17:

1. Build the project:
   ```bash
   cd ~/univrs-Sepah/llvm-backend
   cargo build --release
   ```

2. Run tests:
   ```bash
   cargo test
   ```

3. See [PROGRESS.md](./PROGRESS.md) for next steps in the Native phase implementation.

---

## Additional Resources

- [LLVM Download Page](https://releases.llvm.org/download.html)
- [LLVM Build Documentation](https://llvm.org/docs/CMake.html)
- [Inkwell Documentation](https://thedan64.github.io/inkwell/)
- [llvm-sys Crate](https://crates.io/crates/llvm-sys)

---

*Last updated: 2026-02-09*
