use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

// Webhook signature verification tests.
//
// Validates that the Standard Webhooks signing format used by the Nahook API
// can be correctly produced and verified using native crypto.
//
// Signing spec:
//   base   = "{msgId}.{timestamp}.{payload}"
//   key    = base64_decode(secret_without_whsec_prefix)
//   sig    = "v1," + base64(HMAC-SHA256(key, base))
//   headers: webhook-id, webhook-timestamp, webhook-signature

const TEST_SECRET: &str = "whsec_dGVzdF93ZWJob29rX3NpZ25pbmdfa2V5XzMyYnl0ZXMh";
const MSG_ID: &str = "msg_test_sig_001";
const TIMESTAMP: &str = "1712345678";
const PAYLOAD: &str = r#"{"order_id":"ord_123","amount":49.99}"#;

type HmacSha256 = Hmac<Sha256>;

fn compute_signature(secret: &str, msg_id: &str, timestamp: &str, payload: &str) -> String {
    let raw_secret = secret.strip_prefix("whsec_").unwrap_or(secret);
    let key = BASE64
        .decode(raw_secret)
        .expect("secret must be valid base64");

    let to_sign = format!("{}.{}.{}", msg_id, timestamp, payload);
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC can take key of any size");
    mac.update(to_sign.as_bytes());
    let digest = mac.finalize().into_bytes();

    format!("v1,{}", BASE64.encode(digest))
}

#[test]
fn test_produces_valid_v1_signature() {
    let sig = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let re = regex::Regex::new(r"^v1,[A-Za-z0-9+/]+=*$").unwrap();
    assert!(
        re.is_match(&sig),
        "signature should match v1 format, got: {}",
        sig
    );
}

#[test]
fn test_deterministic_same_inputs_same_signature() {
    let sig1 = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let sig2 = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    assert_eq!(sig1, sig2);
}

#[test]
fn test_rejects_tampered_payload() {
    let original = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let tampered = compute_signature(
        TEST_SECRET,
        MSG_ID,
        TIMESTAMP,
        r#"{"order_id":"ord_123","amount":99.99}"#,
    );
    assert_ne!(original, tampered);
}

#[test]
fn test_rejects_wrong_secret() {
    let original = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let wrong = compute_signature("whsec_d3Jvbmdfc2VjcmV0", MSG_ID, TIMESTAMP, PAYLOAD);
    assert_ne!(original, wrong);
}

#[test]
fn test_rejects_tampered_msg_id() {
    let original = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let tampered = compute_signature(TEST_SECRET, "msg_tampered_id", TIMESTAMP, PAYLOAD);
    assert_ne!(original, tampered);
}

#[test]
fn test_rejects_tampered_timestamp() {
    let original = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let tampered = compute_signature(TEST_SECRET, MSG_ID, "9999999999", PAYLOAD);
    assert_ne!(original, tampered);
}

#[test]
fn test_correct_headers_structure() {
    let sig = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let headers = std::collections::HashMap::from([
        ("content-type", "application/json"),
        ("webhook-id", MSG_ID),
        ("webhook-timestamp", TIMESTAMP),
        ("webhook-signature", sig.as_str()),
    ]);

    assert!(headers["webhook-id"].starts_with("msg_"));
    assert!(headers["webhook-signature"].starts_with("v1,"));
    assert!(headers["webhook-timestamp"]
        .chars()
        .all(|c| c.is_ascii_digit()));
    assert_eq!(headers["content-type"], "application/json");
}

#[test]
fn test_handles_secret_without_prefix() {
    let raw_secret = &TEST_SECRET[6..];
    let with_prefix = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    let without_prefix = compute_signature(raw_secret, MSG_ID, TIMESTAMP, PAYLOAD);
    assert_eq!(with_prefix, without_prefix);
}

#[test]
fn test_matches_known_cross_language_reference_signature() {
    let sig = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, PAYLOAD);
    assert_eq!(sig, "v1,VF1JBS4kdSwmE64FeeiWTgszlPCfaop53x8bwzvHizw=");
}

#[test]
fn test_empty_payload_produces_valid_signature() {
    let sig = compute_signature(TEST_SECRET, MSG_ID, TIMESTAMP, "");
    assert_eq!(sig, "v1,yNFeVvBSs4aZ/sVHHw1MaUWnN1IGK/Ul/16T8aptSJo=");
}

#[test]
fn test_unicode_payload_consistent_across_languages() {
    let sig = compute_signature(
        TEST_SECRET,
        MSG_ID,
        TIMESTAMP,
        r#"{"name":"café","price":"€9.99"}"#,
    );
    assert_eq!(sig, "v1,GcuGAMV9tELnF2rjay6sA8uo5PDPPlhaFi6gKUg06wQ=");
}
