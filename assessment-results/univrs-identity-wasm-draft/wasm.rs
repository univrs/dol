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
    use base64::{engine::general_purpose::STANDARD, Engine};
    let kp = Keypair::generate();
    serde_json::json!({
        "publicKey": kp.public_key().to_base58(),
        "secretKey": STANDARD.encode(kp.to_bytes())
    })
    .to_string()
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

        assert!(verify_signature(&kp.public_key_bytes(), message, &signature));
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
