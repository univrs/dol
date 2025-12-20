// PROSPECTIVE: Unified Identity System
// This spec defines the cryptographic identity foundation for all Univrs components
// Implementation: univrs-identity crate

system univrs.identity @ 0.1.0 {
  requires cryptography.ed25519 >= 0.0.1
  requires encoding.base58 >= 0.0.1
  requires storage.encrypted >= 0.0.1
  
  provides self_sovereign_identity
  provides signature_verification
  provides identity_derivation
}

exegesis {
  Identity is the foundation of trust in a decentralized system.
  
  Core principles:
  1. SELF-SOVEREIGN: No central authority issues or revokes identities
  2. CRYPTOGRAPHIC: Identity = public key, proof = signature
  3. PORTABLE: Same identity works across all Univrs components
  4. PERSISTENT: Identity survives system restarts (encrypted storage)
  
  Ed25519 was chosen for:
  - Speed (15,000+ signatures/sec)
  - Small keys (32 bytes public, 64 bytes signature)
  - Wide adoption (SSH, TLS, libp2p, blockchain)
  - Deterministic signatures (no randomness needed)
  
  This spec is PROTOCOL-AGNOSTIC. The same identity works over:
  - libp2p (P2P network)
  - HTTP (REST APIs)
  - WebSocket (real-time)
  - Future protocols
}

// === CORE PRIMITIVES ===

gene identity.keypair {
  keypair has private_key
  keypair has public_key
  
  private_key is 32 bytes
  private_key is secret
  private_key never transmitted
  
  public_key is 32 bytes
  public_key is derived from private_key
  public_key is the identity
  
  derivation is deterministic
  derivation is one_way
  // Cannot derive private from public
}

gene identity.public_key {
  public_key is 32 bytes
  public_key is globally unique with overwhelming probability
  public_key is the canonical identity
  
  public_key can be encoded as hex       // 64 characters
  public_key can be encoded as base58    // ~44 characters, human-friendly
  public_key can be encoded as base64    // 44 characters
  
  default encoding is base58
}

gene identity.signature {
  signature is 64 bytes
  signature binds message to identity
  signature proves private_key possession
  
  signature is deterministic
  // Same key + same message = same signature
  
  signature is unforgeable without private_key
  // Computational infeasibility assumption
}

// === OPERATIONS ===

trait identity.generate {
  generate creates new keypair
  generate uses cryptographic random source
  generate is non-deterministic
  
  precondition: secure random available
  postcondition: keypair.private_key is random
  postcondition: keypair.public_key is derived
  
  // This is the "birth" of an identity
  is transformation.creation
}

trait identity.sign {
  sign takes private_key
  sign takes message (arbitrary bytes)
  sign returns signature
  
  precondition: private_key is valid
  postcondition: signature verifies with public_key
  
  sign is deterministic
  sign is fast (microseconds)
  
  is transformation.state_transition
}

trait identity.verify {
  verify takes public_key
  verify takes message
  verify takes signature
  verify returns boolean
  
  verify does not require private_key
  verify is stateless
  verify is fast (microseconds)
  
  if signature was created by sign with matching private_key:
    verify returns true
  else:
    verify returns false
  
  is information.sensing
}

// === PERSISTENCE ===

gene identity.stored_keypair {
  stored_keypair has encrypted_private_key
  stored_keypair has public_key           // Stored in plaintext for lookup
  stored_keypair has encryption_params
  
  encryption uses scrypt for key derivation
  encryption uses chacha20-poly1305 for encryption
  
  passphrase is required for decryption
  passphrase is never stored
}

trait identity.save {
  save takes keypair
  save takes path
  save takes passphrase
  
  save derives encryption_key from passphrase
  save encrypts private_key
  save writes to path atomically
  
  precondition: keypair is valid
  precondition: path is writable
  postcondition: file at path contains stored_keypair
  postcondition: file is encrypted
  
  is information.storage.write
}

trait identity.load {
  load takes path
  load takes passphrase
  load returns keypair or error
  
  load reads stored_keypair from path
  load derives encryption_key from passphrase
  load decrypts private_key
  
  if passphrase is wrong:
    return decryption_error
  if file is corrupted:
    return corruption_error
  
  postcondition: returned keypair matches saved keypair
  
  is information.storage.read
}

// === DERIVATION ===

trait identity.derive_peer_id {
  derive_peer_id takes public_key
  derive_peer_id returns libp2p_peer_id
  
  // libp2p uses different encoding of same key
  derivation is deterministic
  derivation is reversible (can extract public_key)
  
  same public_key always produces same peer_id
}

trait identity.derive_display_id {
  derive_display_id takes public_key
  derive_display_id returns string
  
  display_id is human_readable
  display_id is short (first 8 chars of base58)
  display_id is for UI display only
  display_id is not for cryptographic purposes
  
  // e.g., "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy" → "3J98t1Wp"
}

// === CLAIMS AND ATTESTATIONS ===

gene identity.signed_claim {
  claim has signer          // public_key of claimant
  claim has subject         // what the claim is about (may be same as signer)
  claim has predicate       // what is being claimed
  claim has timestamp       // when claim was made
  claim has signature       // proof of signer
  
  claim is verifiable by anyone with signer's public_key
  claim is unforgeable without signer's private_key
}

trait identity.make_claim {
  make_claim takes keypair (signer)
  make_claim takes subject
  make_claim takes predicate
  make_claim returns signed_claim
  
  postcondition: claim.signer == keypair.public_key
  postcondition: claim.signature verifies
  postcondition: claim.timestamp is current
}

trait identity.verify_claim {
  verify_claim takes signed_claim
  verify_claim returns boolean
  
  verify_claim checks signature against signer
  verify_claim checks timestamp is not in future
  verify_claim does not check predicate truth
  // Predicate truth is domain-specific
}

// === CONSTRAINTS ===

constraint identity.uniqueness {
  each public_key is unique with overwhelming probability
  // 2^256 possible keys, collision probability negligible
  
  identity collision would require:
    finding two different private_keys
    that produce same public_key
  // This is computationally infeasible
}

constraint identity.non_transferable {
  identity cannot be transferred
  // You can share private_key, but then both parties "are" that identity
  
  to revoke identity: destroy private_key
  to rotate identity: generate new keypair
}

constraint identity.verification_independence {
  verification does not require network
  verification does not require central authority
  verification requires only: public_key, message, signature
  
  // This enables offline verification
  // This enables decentralized trust
}

// === ERROR TYPES ===

trait identity.errors {
  invalid_signature means signature does not verify
  invalid_public_key means public_key is malformed
  decryption_failed means wrong passphrase or corrupted file
  random_source_unavailable means cannot generate keypair
  
  errors are explicit
  errors are recoverable (except random source)
}

// === USAGE CONTEXTS ===

exegesis {
  Where identity is used in Univrs:
  
  P2P NETWORK (mycelial-dashboard):
  - PeerId derived from public_key
  - Messages signed for authenticity
  - Reputation attestations are signed_claims
  
  ORCHESTRATOR (ai_native_orchestrator):
  - NodeId is public_key
  - Container specs are signed by deployer
  - Scheduling decisions are signed by scheduler
  - State transitions are signed for audit
  
  DASHBOARD:
  - API requests include signed_claims
  - Session tokens are signed_claims with expiry
  
  ECONOMICS (future):
  - Credit transfers are signed by sender
  - Resource commitments are signed_claims
  
  The same keypair provides identity across all contexts.
}
