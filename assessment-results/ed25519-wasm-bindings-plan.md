# Ed25519 WASM Bindings Implementation Plan

## Assessment Date: 2026-01-02

## 1. Current Implementation State

### Location
`~/repos/univrs-identity/`

### Test Status
- **Ed25519 Native: COMPLETE** (45 passing tests verified)
- Tests cover: keypair generation, signing, verification, encoding, storage, claims

### Current Architecture

```
univrs-identity/
├── src/
│   ├── lib.rs           # Main entry, re-exports
│   ├── keypair.rs       # Ed25519 keypair (SigningKey wrapper)
│   ├── signature.rs     # PublicKey, Signature types with encodings
│   ├── storage.rs       # Encrypted storage (scrypt + ChaCha20-Poly1305)
│   ├── claims.rs        # Signed claims/attestations
│   └── error.rs         # Error types
├── Cargo.toml
└── tests pass: 45
```

### Core Dependencies (Current)
```toml
ed25519-dalek = { version = "2.1", features = ["serde", "rand_core"] }
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bs58 = "0.5"
base64 = "0.21"
thiserror = "1.0"
zeroize = { version = "1.7", features = ["derive"] }
chacha20poly1305 = "0.10"
scrypt = "0.11"
```

## 2. WASM Compatibility Issues Identified

### Issue 1: getrandom Requires "js" Feature
**Error when compiling for wasm32-unknown-unknown:**
```
error: the wasm*-unknown-unknown targets are not supported by default,
you may need to enable the "js" feature.
```

**Root Cause:** `rand` -> `rand_core` -> `getrandom` chain requires browser RNG access

**Solution:** Add `getrandom` with "js" feature for WASM target

### Issue 2: Storage Module Uses std::fs
The `storage.rs` module uses filesystem operations that are not available in browser WASM:
- `std::fs::File`
- `std::fs::rename`

**Solution:** Gate storage module behind non-WASM feature flag

### Issue 3: No wasm-bindgen Annotations
Current types lack `#[wasm_bindgen]` annotations needed for JavaScript interop.

**Solution:** Create new `wasm.rs` module with browser-friendly wrappers

## 3. Cargo.toml Modifications

### Proposed Changes

```toml
[package]
name = "univrs-identity"
version = "0.1.0"
edition = "2021"
authors = ["Univrs.io Contributors"]
license = "MIT"
description = "Unified Ed25519 cryptographic identity for Univrs ecosystem"
repository = "https://github.com/univrs/univrs-identity"
keywords = ["ed25519", "cryptography", "identity", "signature", "authentication"]
categories = ["cryptography", "authentication"]

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Ed25519 cryptography
ed25519-dalek = { version = "2.1", features = ["serde", "rand_core"] }

# Random number generation (with WASM support via getrandom)
rand = "0.8"

# WASM browser RNG support - only for wasm32-unknown-unknown target
[target.'cfg(all(target_arch = "wasm32", target_os = "unknown"))'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
wasm-bindgen = "0.2"
js-sys = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Encoding
bs58 = "0.5"
base64 = "0.21"

# Error handling
thiserror = "1.0"

# Secret handling - zeroes memory on drop
zeroize = { version = "1.7", features = ["derive"] }

# Encryption for stored keys (native only - filesystem not available in WASM)
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
chacha20poly1305 = "0.10"
scrypt = "0.11"

[dev-dependencies]
tempfile = "3.10"
wasm-bindgen-test = "0.3"

[features]
default = ["native"]
native = []        # Enable native-only features (storage)
wasm = []          # Enable WASM bindings
libp2p = []        # Enable libp2p PeerId derivation
```

### Key Changes Explained

1. **`crate-type = ["cdylib", "rlib"]`**: Produces both dynamic library (for WASM) and standard Rust library

2. **Target-specific getrandom**: Only enable JS RNG when compiling for browser WASM

3. **Target-specific storage deps**: `chacha20poly1305` and `scrypt` only for native targets

4. **wasm-bindgen-test**: For browser-based testing

## 4. Draft wasm-bindgen Bindings (src/wasm.rs)

```rust
//! WebAssembly bindings for univrs-identity.
//!
//! Provides browser-compatible Ed25519 cryptographic operations.
//!
//! ## Usage in JavaScript/TypeScript
//!
//! ```typescript
//! import init, { WasmKeypair, verify_signature } from 'univrs-identity';
//!
//! async function main() {
//!   await init();
//!
//!   // Generate a new keypair
//!   const keypair = new WasmKeypair();
//!   console.log('Public Key:', keypair.public_key_base58());
//!
//!   // Sign a message
//!   const message = new TextEncoder().encode('Hello, Univrs!');
//!   const signature = keypair.sign(message);
//!
//!   // Verify the signature
//!   const isValid = verify_signature(
//!     keypair.public_key_bytes(),
//!     message,
//!     signature
//!   );
//!   console.log('Valid:', isValid);
//! }
//! ```

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

use crate::keypair::Keypair;
use crate::signature::{PublicKey, Signature};

/// JavaScript error type for WASM bindings.
#[wasm_bindgen]
pub struct WasmError {
    message: String,
}

#[wasm_bindgen]
impl WasmError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<crate::error::IdentityError> for WasmError {
    fn from(err: crate::error::IdentityError) -> Self {
        WasmError {
            message: err.to_string(),
        }
    }
}

/// Ed25519 keypair for browser environments.
///
/// Wraps the native Keypair with JavaScript-friendly API.
#[wasm_bindgen]
pub struct WasmKeypair {
    inner: Keypair,
}

#[wasm_bindgen]
impl WasmKeypair {
    /// Generate a new random keypair.
    ///
    /// Uses the browser's crypto.getRandomValues() for secure randomness.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Keypair::generate(),
        }
    }

    /// Create a keypair from a 32-byte secret key.
    ///
    /// # Arguments
    /// * `secret` - Exactly 32 bytes of secret key material
    ///
    /// # Errors
    /// Returns error if secret is not exactly 32 bytes.
    #[wasm_bindgen]
    pub fn from_bytes(secret: &[u8]) -> Result<WasmKeypair, WasmError> {
        let inner = Keypair::from_slice(secret).map_err(WasmError::from)?;
        Ok(Self { inner })
    }

    /// Get the public key as raw bytes (32 bytes).
    #[wasm_bindgen]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.inner.public_key().as_bytes().to_vec()
    }

    /// Get the public key as Base58 string.
    #[wasm_bindgen]
    pub fn public_key_base58(&self) -> String {
        self.inner.public_key().to_base58()
    }

    /// Get the public key as hex string (64 characters).
    #[wasm_bindgen]
    pub fn public_key_hex(&self) -> String {
        self.inner.public_key().to_hex()
    }

    /// Get a short peer ID (first 8 characters of Base58).
    #[wasm_bindgen]
    pub fn peer_id(&self) -> String {
        self.inner.public_key().to_peer_id()
    }

    /// Sign a message and return the signature bytes (64 bytes).
    ///
    /// # Arguments
    /// * `message` - Arbitrary bytes to sign
    #[wasm_bindgen]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.inner.sign(message).to_bytes().to_vec()
    }

    /// Sign a message and return the signature as Base64.
    #[wasm_bindgen]
    pub fn sign_base64(&self, message: &[u8]) -> String {
        self.inner.sign(message).to_base64()
    }

    /// Sign a message and return the signature as hex.
    #[wasm_bindgen]
    pub fn sign_hex(&self, message: &[u8]) -> String {
        self.inner.sign(message).to_hex()
    }

    /// Verify a signature against this keypair's public key.
    ///
    /// # Arguments
    /// * `message` - The original message bytes
    /// * `signature` - The 64-byte signature
    #[wasm_bindgen]
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        match Signature::from_bytes(signature) {
            Ok(sig) => self.inner.public_key().verify(message, &sig),
            Err(_) => false,
        }
    }

    /// Get the secret key bytes (32 bytes).
    ///
    /// **Security Warning**: Handle with extreme care. Never log or transmit
    /// these bytes without encryption.
    #[wasm_bindgen]
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes().to_vec()
    }
}

impl Default for WasmKeypair {
    fn default() -> Self {
        Self::new()
    }
}

/// A standalone public key for verification operations.
#[wasm_bindgen]
pub struct WasmPublicKey {
    inner: PublicKey,
}

#[wasm_bindgen]
impl WasmPublicKey {
    /// Create from raw bytes (32 bytes).
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<WasmPublicKey, WasmError> {
        let inner = PublicKey::from_bytes(bytes).map_err(WasmError::from)?;
        Ok(Self { inner })
    }

    /// Create from Base58 string.
    #[wasm_bindgen]
    pub fn from_base58(s: &str) -> Result<WasmPublicKey, WasmError> {
        let inner = PublicKey::from_base58(s).map_err(WasmError::from)?;
        Ok(Self { inner })
    }

    /// Create from hex string.
    #[wasm_bindgen]
    pub fn from_hex(s: &str) -> Result<WasmPublicKey, WasmError> {
        let inner = PublicKey::from_hex(s).map_err(WasmError::from)?;
        Ok(Self { inner })
    }

    /// Get as raw bytes.
    #[wasm_bindgen]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    /// Get as Base58 string.
    #[wasm_bindgen]
    pub fn to_base58(&self) -> String {
        self.inner.to_base58()
    }

    /// Get as hex string.
    #[wasm_bindgen]
    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    /// Get short peer ID.
    #[wasm_bindgen]
    pub fn to_peer_id(&self) -> String {
        self.inner.to_peer_id()
    }

    /// Verify a signature.
    #[wasm_bindgen]
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        match Signature::from_bytes(signature) {
            Ok(sig) => self.inner.verify(message, &sig),
            Err(_) => false,
        }
    }
}

// =============================================================================
// Standalone Functions (for functional-style JavaScript usage)
// =============================================================================

/// Generate a new random keypair and return as JSON.
///
/// Returns: `{ "publicKey": "base58...", "secretKey": "base64..." }`
#[wasm_bindgen]
pub fn generate_keypair_json() -> String {
    let kp = Keypair::generate();
    serde_json::json!({
        "publicKey": kp.public_key().to_base58(),
        "secretKey": base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            kp.to_bytes()
        )
    }).to_string()
}

/// Verify a signature given public key bytes, message, and signature bytes.
///
/// # Arguments
/// * `public_key` - 32-byte public key
/// * `message` - Original message bytes
/// * `signature` - 64-byte signature
#[wasm_bindgen]
pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let pk = match PublicKey::from_bytes(public_key) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let sig = match Signature::from_bytes(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    pk.verify(message, &sig)
}

/// Sign a message given secret key bytes and message.
///
/// # Arguments
/// * `secret_key` - 32-byte secret key
/// * `message` - Message bytes to sign
///
/// Returns 64-byte signature or empty array on error.
#[wasm_bindgen]
pub fn sign_message(secret_key: &[u8], message: &[u8]) -> Vec<u8> {
    match Keypair::from_slice(secret_key) {
        Ok(kp) => kp.sign(message).to_bytes().to_vec(),
        Err(_) => Vec::new(),
    }
}

/// Derive public key from secret key.
///
/// # Arguments
/// * `secret_key` - 32-byte secret key
///
/// Returns 32-byte public key or empty array on error.
#[wasm_bindgen]
pub fn derive_public_key(secret_key: &[u8]) -> Vec<u8> {
    match Keypair::from_slice(secret_key) {
        Ok(kp) => kp.public_key().as_bytes().to_vec(),
        Err(_) => Vec::new(),
    }
}

// =============================================================================
// WASM Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_keypair_generation() {
        let kp = WasmKeypair::new();
        assert_eq!(kp.public_key_bytes().len(), 32);
    }

    #[wasm_bindgen_test]
    fn test_sign_and_verify() {
        let kp = WasmKeypair::new();
        let message = b"Hello, WASM!";
        let signature = kp.sign(message);

        assert_eq!(signature.len(), 64);
        assert!(kp.verify(message, &signature));
    }

    #[wasm_bindgen_test]
    fn test_wrong_message_fails() {
        let kp = WasmKeypair::new();
        let signature = kp.sign(b"original message");

        assert!(!kp.verify(b"different message", &signature));
    }

    #[wasm_bindgen_test]
    fn test_standalone_verify() {
        let kp = WasmKeypair::new();
        let message = b"test";
        let signature = kp.sign(message);

        assert!(verify_signature(
            &kp.public_key_bytes(),
            message,
            &signature
        ));
    }

    #[wasm_bindgen_test]
    fn test_public_key_from_bytes() {
        let kp = WasmKeypair::new();
        let pk_bytes = kp.public_key_bytes();

        let pk = WasmPublicKey::from_bytes(&pk_bytes).unwrap();
        assert_eq!(pk.to_bytes(), pk_bytes);
    }

    #[wasm_bindgen_test]
    fn test_public_key_encodings() {
        let kp = WasmKeypair::new();
        let b58 = kp.public_key_base58();

        let pk = WasmPublicKey::from_base58(&b58).unwrap();
        assert_eq!(pk.to_base58(), b58);
    }

    #[wasm_bindgen_test]
    fn test_keypair_from_bytes_roundtrip() {
        let kp1 = WasmKeypair::new();
        let secret = kp1.secret_key_bytes();

        let kp2 = WasmKeypair::from_bytes(&secret).unwrap();
        assert_eq!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }
}
```

## 5. lib.rs Modifications

Add conditional compilation for WASM module:

```rust
// At the end of lib.rs, add:

#[cfg(feature = "wasm")]
pub mod wasm;

// Gate storage behind non-WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;

#[cfg(target_arch = "wasm32")]
pub mod storage {
    //! Storage is not available in browser WASM environments.
    //! Use browser localStorage or IndexedDB directly from JavaScript.
}
```

## 6. Browser Test Plan

### 6.1 Build Process

```bash
# Install wasm-pack if not present
cargo install wasm-pack

# Build for browser
wasm-pack build --target web --features wasm

# Build for bundler (webpack, etc.)
wasm-pack build --target bundler --features wasm

# Build for Node.js
wasm-pack build --target nodejs --features wasm
```

### 6.2 Test Execution

```bash
# Run WASM tests in headless browser
wasm-pack test --headless --chrome --features wasm
wasm-pack test --headless --firefox --features wasm

# Run WASM tests in Node.js
wasm-pack test --node --features wasm
```

### 6.3 Manual Browser Test HTML

Create `tests/browser/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>univrs-identity WASM Test</title>
</head>
<body>
    <h1>Ed25519 WASM Test</h1>
    <div id="results"></div>

    <script type="module">
        import init, {
            WasmKeypair,
            WasmPublicKey,
            verify_signature,
            sign_message,
            generate_keypair_json
        } from './pkg/univrs_identity.js';

        async function runTests() {
            await init();
            const results = document.getElementById('results');
            let passed = 0;
            let failed = 0;

            function test(name, fn) {
                try {
                    fn();
                    results.innerHTML += `<p style="color:green">PASS: ${name}</p>`;
                    passed++;
                } catch (e) {
                    results.innerHTML += `<p style="color:red">FAIL: ${name} - ${e}</p>`;
                    failed++;
                }
            }

            // Test 1: Keypair generation
            test('Keypair generation', () => {
                const kp = new WasmKeypair();
                if (kp.public_key_bytes().length !== 32) {
                    throw new Error('Public key should be 32 bytes');
                }
            });

            // Test 2: Sign and verify
            test('Sign and verify', () => {
                const kp = new WasmKeypair();
                const message = new TextEncoder().encode('Hello, WASM!');
                const signature = kp.sign(message);

                if (signature.length !== 64) {
                    throw new Error('Signature should be 64 bytes');
                }
                if (!kp.verify(message, signature)) {
                    throw new Error('Verification failed');
                }
            });

            // Test 3: Wrong message fails verification
            test('Wrong message fails', () => {
                const kp = new WasmKeypair();
                const signature = kp.sign(new TextEncoder().encode('original'));

                if (kp.verify(new TextEncoder().encode('different'), signature)) {
                    throw new Error('Should have failed verification');
                }
            });

            // Test 4: Standalone verify function
            test('Standalone verify', () => {
                const kp = new WasmKeypair();
                const message = new TextEncoder().encode('test');
                const signature = kp.sign(message);

                if (!verify_signature(kp.public_key_bytes(), message, signature)) {
                    throw new Error('Standalone verify failed');
                }
            });

            // Test 5: Public key from Base58
            test('Public key Base58 roundtrip', () => {
                const kp = new WasmKeypair();
                const b58 = kp.public_key_base58();
                const pk = WasmPublicKey.from_base58(b58);

                if (pk.to_base58() !== b58) {
                    throw new Error('Base58 roundtrip failed');
                }
            });

            // Test 6: JSON keypair generation
            test('JSON keypair generation', () => {
                const json = generate_keypair_json();
                const parsed = JSON.parse(json);

                if (!parsed.publicKey || !parsed.secretKey) {
                    throw new Error('JSON missing fields');
                }
            });

            // Test 7: Keypair restoration from bytes
            test('Keypair restoration', () => {
                const kp1 = new WasmKeypair();
                const secret = kp1.secret_key_bytes();
                const kp2 = WasmKeypair.from_bytes(secret);

                const pk1 = kp1.public_key_base58();
                const pk2 = kp2.public_key_base58();

                if (pk1 !== pk2) {
                    throw new Error('Restored keypair mismatch');
                }
            });

            // Summary
            results.innerHTML += `<hr><p><strong>Results: ${passed} passed, ${failed} failed</strong></p>`;
        }

        runTests();
    </script>
</body>
</html>
```

### 6.4 TypeScript Definitions

wasm-pack automatically generates TypeScript definitions. Expected output in `pkg/univrs_identity.d.ts`:

```typescript
export class WasmKeypair {
  constructor();
  static from_bytes(secret: Uint8Array): WasmKeypair;
  public_key_bytes(): Uint8Array;
  public_key_base58(): string;
  public_key_hex(): string;
  peer_id(): string;
  sign(message: Uint8Array): Uint8Array;
  sign_base64(message: Uint8Array): string;
  sign_hex(message: Uint8Array): string;
  verify(message: Uint8Array, signature: Uint8Array): boolean;
  secret_key_bytes(): Uint8Array;
}

export class WasmPublicKey {
  static from_bytes(bytes: Uint8Array): WasmPublicKey;
  static from_base58(s: string): WasmPublicKey;
  static from_hex(s: string): WasmPublicKey;
  to_bytes(): Uint8Array;
  to_base58(): string;
  to_hex(): string;
  to_peer_id(): string;
  verify(message: Uint8Array, signature: Uint8Array): boolean;
}

export function generate_keypair_json(): string;
export function verify_signature(public_key: Uint8Array, message: Uint8Array, signature: Uint8Array): boolean;
export function sign_message(secret_key: Uint8Array, message: Uint8Array): Uint8Array;
export function derive_public_key(secret_key: Uint8Array): Uint8Array;
```

## 7. Implementation Steps

### Phase 1: Cargo.toml Updates
1. Add wasm-bindgen dependency for wasm32 target
2. Add getrandom with "js" feature for wasm32 target
3. Gate storage dependencies behind native feature
4. Add wasm-bindgen-test to dev-dependencies
5. Update crate-type to include cdylib

### Phase 2: Source Code Changes
1. Create `src/wasm.rs` with bindings
2. Update `src/lib.rs` with conditional compilation
3. Gate `storage.rs` behind non-WASM target

### Phase 3: Testing
1. Run native tests to ensure no regression
2. Build with `wasm-pack build`
3. Run `wasm-pack test --headless --chrome`
4. Test manual HTML page in browser

### Phase 4: Documentation
1. Add WASM usage examples to README
2. Document browser limitations (no storage)
3. Add npm package.json for publishing

## 8. Alternative: Native Browser Crypto API

As of 2025, all major browsers support Ed25519 natively via SubtleCrypto:
- Chrome 137+ (May 2025)
- Firefox 129+ (August 2024)
- Safari 17.0+

Consider adding a "native-crypto" feature that uses Web Crypto API instead of WASM for better security (non-extractable keys) and smaller bundle size.

```typescript
// Alternative: Use browser native Ed25519
async function useNativeCrypto() {
    const keyPair = await crypto.subtle.generateKey(
        { name: 'Ed25519' },
        true,  // extractable
        ['sign', 'verify']
    );

    const signature = await crypto.subtle.sign(
        { name: 'Ed25519' },
        keyPair.privateKey,
        message
    );

    const isValid = await crypto.subtle.verify(
        { name: 'Ed25519' },
        keyPair.publicKey,
        signature,
        message
    );
}
```

## 9. File Checklist

| File | Action | Purpose |
|------|--------|---------|
| Cargo.toml | Modify | Add WASM dependencies |
| src/lib.rs | Modify | Add conditional wasm module |
| src/wasm.rs | Create | WASM bindings |
| src/storage.rs | Modify | Gate behind non-WASM |
| tests/browser/index.html | Create | Manual browser test |
| README.md | Update | Add WASM documentation |
