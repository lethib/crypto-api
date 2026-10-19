use serde_json::Value;

use crate::signing::canonicalize::canonicalize;

pub mod canonicalize;
pub mod hmac_sha256;

pub trait Signer: Send + Sync {
    fn sign(&self, message: &[u8]) -> String;
    fn verify(&self, message: &[u8], signature: &str) -> bool;
}

pub fn sign_value(signer: &dyn Signer, value: &Value) -> String {
    let canonicalized_value = canonicalize(value);
    signer.sign(canonicalized_value.as_bytes())
}

pub fn verify_value(signer: &dyn Signer, value: &Value, signature: &str) -> bool {
    let canonicalized_value = canonicalize(value);
    signer.verify(canonicalized_value.as_bytes(), signature)
}
