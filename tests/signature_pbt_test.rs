//! Property-based tests for webhook signature signing and verification.
//!
//! Uses proptest to generate arbitrary payloads, secrets, message IDs, and
//! timestamps, then asserts invariants that must hold for all inputs.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use proptest::prelude::*;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Sign a payload using the Standard Webhooks format.
fn sign(secret: &[u8], msg_id: &str, timestamp: &str, payload: &str) -> String {
    let to_sign = format!("{}.{}.{}", msg_id, timestamp, payload);
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    mac.update(to_sign.as_bytes());
    let digest = mac.finalize().into_bytes();
    format!("v1,{}", BASE64.encode(digest))
}

/// Verify a signature against a payload.
fn verify(secret: &[u8], msg_id: &str, timestamp: &str, payload: &str, signature: &str) -> bool {
    let expected = sign(secret, msg_id, timestamp, payload);
    expected == signature
}

proptest! {
    // Property 1: sign then verify roundtrip always succeeds.
    #[test]
    fn prop_sign_verify_roundtrip(
        secret in prop::collection::vec(1u8..=255, 16..=64),
        msg_id in "[a-z0-9_]{5,30}",
        timestamp in "[0-9]{10}",
        payload in ".*",
    ) {
        let sig = sign(&secret, &msg_id, &timestamp, &payload);
        prop_assert!(
            verify(&secret, &msg_id, &timestamp, &payload, &sig),
            "sign-then-verify must always succeed"
        );
    }

    // Property 2: tampered payload fails verification.
    #[test]
    fn prop_tampered_payload_fails(
        secret in prop::collection::vec(1u8..=255, 16..=64),
        msg_id in "[a-z0-9_]{5,30}",
        timestamp in "[0-9]{10}",
        payload in ".{1,200}",
        tamper_byte in 0u8..=255u8,
        tamper_pos in 0usize..200,
    ) {
        let sig = sign(&secret, &msg_id, &timestamp, &payload);

        // Tamper the payload by changing one byte
        let mut tampered_bytes = payload.as_bytes().to_vec();
        let idx = tamper_pos % tampered_bytes.len();
        if tampered_bytes[idx] != tamper_byte {
            tampered_bytes[idx] = tamper_byte;
            // Only check if the tampered bytes form valid UTF-8
            if let Ok(tampered) = String::from_utf8(tampered_bytes) {
                if tampered != payload {
                    prop_assert!(
                        !verify(&secret, &msg_id, &timestamp, &tampered, &sig),
                        "tampered payload must fail verification"
                    );
                }
            }
        }
    }

    // Property 3: wrong secret fails verification.
    #[test]
    fn prop_wrong_secret_fails(
        secret1 in prop::collection::vec(1u8..=255, 16..=64),
        secret2 in prop::collection::vec(1u8..=255, 16..=64),
        msg_id in "[a-z0-9_]{5,30}",
        timestamp in "[0-9]{10}",
        payload in ".*",
    ) {
        prop_assume!(secret1 != secret2);
        let sig = sign(&secret1, &msg_id, &timestamp, &payload);
        prop_assert!(
            !verify(&secret2, &msg_id, &timestamp, &payload, &sig),
            "wrong secret must fail verification"
        );
    }

    // Property 4: signing is deterministic — same inputs always produce the same output.
    #[test]
    fn prop_deterministic(
        secret in prop::collection::vec(1u8..=255, 16..=64),
        msg_id in "[a-z0-9_]{5,30}",
        timestamp in "[0-9]{10}",
        payload in ".*",
    ) {
        let sig1 = sign(&secret, &msg_id, &timestamp, &payload);
        let sig2 = sign(&secret, &msg_id, &timestamp, &payload);
        prop_assert_eq!(sig1, sig2, "sign() must be deterministic");
    }
}
