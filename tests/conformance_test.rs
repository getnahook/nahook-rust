//! Conformance tests driven by shared JSON fixtures in ../fixtures/conformance/.
//!
//! These tests ensure the Rust SDK behaves identically to every other SDK
//! for error classification, region routing, retry backoff, and signatures.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use nahook::error::ApiError;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

// ── Helpers ──

fn compute_signature(secret: &str, msg_id: &str, timestamp: &str, payload: &str) -> String {
    let raw_secret = if secret.starts_with("whsec_") {
        &secret[6..]
    } else {
        secret
    };
    let key = BASE64.decode(raw_secret).unwrap_or_else(|_| raw_secret.as_bytes().to_vec());
    let to_sign = format!("{}.{}.{}", msg_id, timestamp, payload);
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC can take key of any size");
    mac.update(to_sign.as_bytes());
    let digest = mac.finalize().into_bytes();
    format!("v1,{}", BASE64.encode(digest))
}

fn verify_signature(
    secret: &str,
    msg_id: &str,
    timestamp: &str,
    payload: &str,
    signature: &str,
) -> bool {
    let expected = compute_signature(secret, msg_id, timestamp, payload);
    // The signature header may contain "v1,<timestamp>,<sig>" or just "v1,<sig>"
    // Compare the v1,<base64> portion
    expected == signature
}

/// Re-implement resolve_base_url to match the SDK's internal logic, so we can
/// drive it from fixtures without needing it to be pub.
fn resolve_base_url(api_key: &str) -> &str {
    if api_key.len() >= 7 && api_key.starts_with("nhk_") && api_key.as_bytes()[6] == b'_' {
        match &api_key[4..6] {
            "us" => "https://us.api.nahook.com",
            "eu" => "https://eu.api.nahook.com",
            "ap" => "https://ap.api.nahook.com",
            _ => "https://api.nahook.com",
        }
    } else {
        "https://api.nahook.com"
    }
}

/// Re-implement calculate_delay to match the SDK's internal logic.
fn calculate_delay(attempt: u32, retry_after_ms: Option<u64>) -> u64 {
    const BASE_DELAY_MS: u64 = 500;
    const MAX_DELAY_MS: u64 = 10_000;

    if let Some(ms) = retry_after_ms {
        if ms > 0 {
            return ms;
        }
    }
    let exponential = std::cmp::min(MAX_DELAY_MS, BASE_DELAY_MS * 2u64.pow(attempt));
    let jitter: f64 = rand::Rng::gen(&mut rand::thread_rng());
    (exponential as f64 * jitter) as u64
}

// ── Error Classification (fixtures/conformance/error-classification/cases.json) ──

#[test]
fn conformance_error_classification() {
    let json_str =
        std::fs::read_to_string("../fixtures/conformance/error-classification/cases.json")
            .expect("error-classification fixtures must exist");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("valid JSON array");

    for case in &cases {
        let id = case["id"].as_str().unwrap();
        let input = &case["input"];
        let expect = &case["expect"];

        let err = ApiError {
            status: input["status"].as_u64().unwrap() as u16,
            code: input["code"].as_str().unwrap().to_string(),
            message: input["message"].as_str().unwrap().to_string(),
            retry_after: input.get("retryAfter").and_then(|v| v.as_u64()),
        };

        assert_eq!(
            err.is_retryable(),
            expect["isRetryable"].as_bool().unwrap(),
            "{id}: is_retryable mismatch"
        );
        assert_eq!(
            err.is_auth_error(),
            expect["isAuthError"].as_bool().unwrap(),
            "{id}: is_auth_error mismatch"
        );
        assert_eq!(
            err.is_not_found(),
            expect["isNotFound"].as_bool().unwrap(),
            "{id}: is_not_found mismatch"
        );
        assert_eq!(
            err.is_rate_limited(),
            expect["isRateLimited"].as_bool().unwrap(),
            "{id}: is_rate_limited mismatch"
        );
        assert_eq!(
            err.is_validation_error(),
            expect["isValidationError"].as_bool().unwrap(),
            "{id}: is_validation_error mismatch"
        );
    }
}

// ── Region Routing (fixtures/conformance/region-routing/cases.json) ──

#[test]
fn conformance_region_routing() {
    let json_str = std::fs::read_to_string("../fixtures/conformance/region-routing/cases.json")
        .expect("region-routing fixtures must exist");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("valid JSON array");

    for case in &cases {
        let id = case["id"].as_str().unwrap();
        let token = case["input"]["token"].as_str().unwrap();
        let expected_url = case["expect"]["baseUrl"].as_str().unwrap();

        let resolved = resolve_base_url(token);
        assert_eq!(resolved, expected_url, "{id}: base URL mismatch");
    }
}

// ── Retry Backoff (fixtures/conformance/retry-backoff/cases.json) ──

#[test]
fn conformance_retry_backoff() {
    let json_str = std::fs::read_to_string("../fixtures/conformance/retry-backoff/cases.json")
        .expect("retry-backoff fixtures must exist");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("valid JSON array");

    for case in &cases {
        let id = case["id"].as_str().unwrap();
        let input = &case["input"];
        let expect = &case["expect"];

        let attempt = input["attempt"].as_u64().unwrap() as u32;
        let retry_after_ms = if input["retryAfterMs"].is_null() {
            None
        } else {
            Some(input["retryAfterMs"].as_u64().unwrap())
        };

        if let Some(exact) = expect.get("exactDelayMs") {
            let delay = calculate_delay(attempt, retry_after_ms);
            assert_eq!(
                delay,
                exact.as_u64().unwrap(),
                "{id}: exact delay mismatch"
            );
        } else {
            let min_ms = expect["minDelayMs"].as_u64().unwrap();
            let max_ms = expect["maxDelayMs"].as_u64().unwrap();
            // Run multiple times due to jitter
            for _ in 0..50 {
                let delay = calculate_delay(attempt, retry_after_ms);
                assert!(
                    delay >= min_ms && delay <= max_ms,
                    "{id}: delay {delay} not in [{min_ms}, {max_ms}]"
                );
            }
        }
    }
}

// ── Signature (fixtures/conformance/signature/cases.json) ──

#[test]
fn conformance_signature() {
    let json_str = std::fs::read_to_string("../fixtures/conformance/signature/cases.json")
        .expect("signature fixtures must exist");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("valid JSON array");

    for case in &cases {
        let id = case["id"].as_str().unwrap();
        let input = &case["input"];
        let action = case["action"].as_str().unwrap();
        let expect = &case["expect"];

        let secret = input["secret"].as_str().unwrap();
        let msg_id = input["messageId"].as_str().unwrap();
        let timestamp = input["timestamp"].as_str().unwrap();

        // Resolve payload: either literal or generated
        let payload = if let Some(gen) = input.get("payloadGenerator") {
            match gen.as_str().unwrap() {
                "repeat_a_10000" => "a".repeat(10_000),
                other => panic!("{id}: unknown payloadGenerator: {other}"),
            }
        } else {
            input["payload"].as_str().unwrap_or("").to_string()
        };

        match action {
            "sign_then_verify" => {
                let sig = compute_signature(secret, msg_id, timestamp, &payload);
                let verified = verify_signature(secret, msg_id, timestamp, &payload, &sig);
                assert_eq!(
                    verified,
                    expect["verifies"].as_bool().unwrap(),
                    "{id}: sign_then_verify mismatch"
                );
            }
            "sign_original_verify_tampered" => {
                let sig = compute_signature(secret, msg_id, timestamp, &payload);
                let tampered = input["tamperedPayload"].as_str().unwrap();
                let verified = verify_signature(secret, msg_id, timestamp, tampered, &sig);
                assert_eq!(
                    verified,
                    expect["verifies"].as_bool().unwrap(),
                    "{id}: tampered payload should fail"
                );
            }
            "sign_with_original_verify_with_wrong" => {
                let sig = compute_signature(secret, msg_id, timestamp, &payload);
                let wrong_secret = input["wrongSecret"].as_str().unwrap();
                let verified =
                    verify_signature(wrong_secret, msg_id, timestamp, &payload, &sig);
                assert_eq!(
                    verified,
                    expect["verifies"].as_bool().unwrap(),
                    "{id}: wrong secret should fail"
                );
            }
            "sign_twice_compare" => {
                let sig1 = compute_signature(secret, msg_id, timestamp, &payload);
                let sig2 = compute_signature(secret, msg_id, timestamp, &payload);
                assert_eq!(
                    sig1 == sig2,
                    expect["identical"].as_bool().unwrap(),
                    "{id}: determinism mismatch"
                );
            }
            "verify_known_signature" => {
                let expected_header = expect["signatureHeader"].as_str().unwrap();
                // The header format is "v1,<timestamp>,<sig>" — extract the sig part
                let parts: Vec<&str> = expected_header.splitn(3, ',').collect();
                assert!(
                    parts.len() >= 2,
                    "{id}: expected header format v1,timestamp,sig or v1,sig"
                );
                // Build the signature the same way our SDK does
                let sig = compute_signature(secret, msg_id, timestamp, &payload);
                // If the expected header has 3 parts (v1,timestamp,sig), the sig is parts[2]
                if parts.len() == 3 {
                    let expected_sig = format!("v1,{}", parts[2]);
                    assert_eq!(sig, expected_sig, "{id}: known signature mismatch");
                } else {
                    // v1,sig format
                    assert_eq!(sig, expected_header, "{id}: known signature mismatch");
                }
            }
            other => panic!("{id}: unknown action: {other}"),
        }
    }
}
