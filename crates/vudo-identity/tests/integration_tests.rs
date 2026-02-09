//! Integration tests for vudo-identity

use chrono::Utc;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use vudo_identity::{
    Capability, DeviceIdentity, Did, DidResolver, MasterIdentity, Ucan,
};
use x25519_dalek::{PublicKey, StaticSecret};

#[tokio::test]
async fn test_complete_identity_workflow() {
    // 1. Create master identity
    let mut master = MasterIdentity::generate("Alice").await.unwrap();
    assert_eq!(master.name, "Alice");

    // 2. Create device
    let mut device = DeviceIdentity::generate("Alice's Phone").await.unwrap();
    assert_eq!(device.device_name(), "Alice's Phone");

    // 3. Link device to master
    let master_key = master.signing_key();
    let link = master
        .link_device(
            device.device_name().to_string(),
            device.did().clone(),
            &master_key,
        )
        .await
        .unwrap();

    // 4. Update device with authorization
    device.link_to_master(master.did.clone(), link.authorization.clone());
    assert!(device.is_linked());

    // 5. Verify device authorization
    assert!(device.verify_authorization().is_ok());

    // 6. Create app UCAN from device
    let app_key = SigningKey::generate(&mut OsRng);
    let app_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let app_did = Did::from_keys(app_key.verifying_key(), &app_enc).unwrap();

    let device_key = device.signing_key();
    let app_ucan = link
        .authorization
        .delegate(
            app_did.clone(),
            vec![Capability::new("vudo://myapp/data", "read")],
            Utc::now().timestamp() as u64 + 1800,
            &device_key,
        )
        .unwrap();

    // 7. Verify app UCAN (includes delegation chain)
    assert!(app_ucan.verify().is_ok());
}

#[tokio::test]
async fn test_device_revocation() {
    let mut master = MasterIdentity::generate("Bob").await.unwrap();
    let device = DeviceIdentity::generate("Bob's Laptop").await.unwrap();

    let master_key = master.signing_key();
    master
        .link_device(
            device.device_name().to_string(),
            device.did().clone(),
            &master_key,
        )
        .await
        .unwrap();

    assert!(!master.is_device_revoked(device.did()));

    // Revoke device
    master
        .revoke_device(
            device.did(),
            Some("Device lost".to_string()),
            &master_key,
        )
        .await
        .unwrap();

    assert!(master.is_device_revoked(device.did()));
}

#[tokio::test]
async fn test_key_rotation_with_grace_period() {
    let mut master = MasterIdentity::generate("Charlie").await.unwrap();
    let old_did = master.did.clone();

    // Rotate key
    let new_key = SigningKey::generate(&mut OsRng);
    let new_encryption = StaticSecret::random_from_rng(&mut OsRng);

    let rotation = master.rotate_key(new_key, new_encryption).await.unwrap();

    // Verify rotation
    assert!(rotation.verify().is_ok());
    assert_ne!(old_did, master.did);
    assert!(rotation.in_grace_period());
    assert_eq!(rotation.old_did, old_did);
    assert_eq!(rotation.new_did, master.did);
}

#[tokio::test]
async fn test_ucan_delegation_chain() {
    // Create three DIDs: Alice -> Bob -> Carol
    let alice_key = SigningKey::generate(&mut OsRng);
    let alice_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let alice_did = Did::from_keys(alice_key.verifying_key(), &alice_enc).unwrap();

    let bob_key = SigningKey::generate(&mut OsRng);
    let bob_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let bob_did = Did::from_keys(bob_key.verifying_key(), &bob_enc).unwrap();

    let carol_key = SigningKey::generate(&mut OsRng);
    let carol_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let carol_did = Did::from_keys(carol_key.verifying_key(), &carol_enc).unwrap();

    // Alice grants Bob full access
    let alice_to_bob = Ucan::new(
        alice_did.clone(),
        bob_did.clone(),
        vec![Capability::wildcard("vudo://myapp/")],
        Utc::now().timestamp() as u64 + 3600,
        None,
        None,
        vec![],
    )
    .sign(&alice_key)
    .unwrap();

    // Bob delegates read access to Carol
    let bob_to_carol = alice_to_bob
        .delegate(
            carol_did.clone(),
            vec![Capability::new("vudo://myapp/data", "read")],
            Utc::now().timestamp() as u64 + 1800,
            &bob_key,
        )
        .unwrap();

    // Verify entire chain
    assert!(bob_to_carol.verify().is_ok());

    // Verify Carol has the capability
    assert!(bob_to_carol
        .grants_to(&carol_did, &[Capability::new("vudo://myapp/data", "read")])
        .unwrap());
}

#[tokio::test]
async fn test_did_resolution() {
    let resolver = DidResolver::new();

    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);
    let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

    // First resolution
    let doc1 = resolver.resolve(&did).await.unwrap();
    assert_eq!(doc1.id, did.as_str());

    // Second resolution (from cache)
    let doc2 = resolver.resolve(&did).await.unwrap();
    assert_eq!(doc1.id, doc2.id);

    // Verify cache
    assert_eq!(resolver.cache_size(), 1);
}

#[tokio::test]
async fn test_ucan_expiration() {
    let issuer_key = SigningKey::generate(&mut OsRng);
    let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc).unwrap();

    let audience_key = SigningKey::generate(&mut OsRng);
    let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc).unwrap();

    // Create expired UCAN
    let ucan = Ucan::new(
        issuer_did,
        audience_did,
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 - 1, // Already expired
        None,
        None,
        vec![],
    )
    .sign(&issuer_key)
    .unwrap();

    // Verification should fail
    assert!(ucan.verify().is_err());
}

#[tokio::test]
async fn test_capability_matching() {
    let cap_wildcard = Capability::wildcard("vudo://myapp/");
    let cap_specific = Capability::new("vudo://myapp/data", "read");

    assert!(cap_wildcard.matches(&cap_specific));
    assert!(!cap_specific.matches(&cap_wildcard));

    let cap_write = Capability::new("vudo://myapp/data", "write");
    assert!(cap_wildcard.matches(&cap_write));

    let cap_other_app = Capability::new("vudo://otherapp/data", "read");
    assert!(!cap_wildcard.matches(&cap_other_app));
}

#[tokio::test]
async fn test_revocation_list_operations() {
    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);
    let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

    let mut master = MasterIdentity::generate("Dave").await.unwrap();
    let master_key = master.signing_key();

    // Initial state
    assert_eq!(master.revocations.version, 0);
    assert!(master.revocations.revocations.is_empty());

    // Add revocation
    master
        .revocations
        .revoke(did.to_string(), Some("Test revocation".to_string()), &master_key)
        .unwrap();

    assert_eq!(master.revocations.version, 1);
    assert_eq!(master.revocations.revocations.len(), 1);
    assert!(master.revocations.is_revoked(&did.to_string()));

    // Verify signature
    assert!(master.revocations.verify().is_ok());
}

#[tokio::test]
async fn test_multiple_device_linking() {
    let mut master = MasterIdentity::generate("Eve").await.unwrap();
    let master_key = master.signing_key();

    let device1 = DeviceIdentity::generate("Phone").await.unwrap();
    let device2 = DeviceIdentity::generate("Laptop").await.unwrap();
    let device3 = DeviceIdentity::generate("Tablet").await.unwrap();

    master
        .link_device("Phone".to_string(), device1.did().clone(), &master_key)
        .await
        .unwrap();
    master
        .link_device("Laptop".to_string(), device2.did().clone(), &master_key)
        .await
        .unwrap();
    master
        .link_device("Tablet".to_string(), device3.did().clone(), &master_key)
        .await
        .unwrap();

    assert_eq!(master.devices.len(), 3);
    assert!(master.devices.iter().all(|d| !d.revoked));
}

#[tokio::test]
async fn test_ucan_encoding_decoding() {
    let issuer_key = SigningKey::generate(&mut OsRng);
    let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc).unwrap();

    let audience_key = SigningKey::generate(&mut OsRng);
    let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc).unwrap();

    let ucan = Ucan::new(
        issuer_did.clone(),
        audience_did.clone(),
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 + 3600,
        None,
        None,
        vec![],
    )
    .sign(&issuer_key)
    .unwrap();

    // Encode as JWT
    let jwt = ucan.encode().unwrap();
    assert!(jwt.contains('.'));

    // Decode from JWT
    let decoded = Ucan::decode(&jwt).unwrap();
    assert_eq!(decoded.iss, issuer_did);
    assert_eq!(decoded.aud, audience_did);
    assert_eq!(decoded.att.len(), 1);

    // Verify decoded UCAN
    assert!(decoded.verify().is_ok());
}

#[tokio::test]
async fn test_did_document_generation() {
    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);
    let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

    let doc = did.to_document();

    assert_eq!(doc.context, "https://www.w3.org/ns/did/v1");
    assert_eq!(doc.id, did.as_str());
    assert_eq!(doc.authentication.len(), 1);
    assert_eq!(doc.key_agreement.len(), 1);

    assert_eq!(
        doc.authentication[0].method_type,
        "Ed25519VerificationKey2020"
    );
    assert_eq!(
        doc.key_agreement[0].method_type,
        "X25519KeyAgreementKey2020"
    );
}

#[tokio::test]
async fn test_did_parse_roundtrip() {
    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);

    let did1 = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
    let did_str = did1.as_str();

    let did2 = Did::parse(did_str).unwrap();

    assert_eq!(did1, did2);
    assert_eq!(did1.verification_key, did2.verification_key);
    assert_eq!(
        did1.encryption_key.as_bytes(),
        did2.encryption_key.as_bytes()
    );
}

#[tokio::test]
async fn test_insufficient_delegation() {
    let alice_key = SigningKey::generate(&mut OsRng);
    let alice_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let alice_did = Did::from_keys(alice_key.verifying_key(), &alice_enc).unwrap();

    let bob_key = SigningKey::generate(&mut OsRng);
    let bob_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let bob_did = Did::from_keys(bob_key.verifying_key(), &bob_enc).unwrap();

    let carol_key = SigningKey::generate(&mut OsRng);
    let carol_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let carol_did = Did::from_keys(carol_key.verifying_key(), &carol_enc).unwrap();

    // Alice grants Bob read-only access
    let alice_to_bob = Ucan::new(
        alice_did.clone(),
        bob_did.clone(),
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 + 3600,
        None,
        None,
        vec![],
    )
    .sign(&alice_key)
    .unwrap();

    // Bob tries to delegate write access to Carol (should fail)
    let result = alice_to_bob.delegate(
        carol_did.clone(),
        vec![Capability::new("vudo://myapp/data", "write")],
        Utc::now().timestamp() as u64 + 1800,
        &bob_key,
    );

    assert!(result.is_err());
}
