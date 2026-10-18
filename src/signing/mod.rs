pub mod canonicalize;
pub mod hmac_sha256;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> String;
    fn verify(&self, message: &[u8], signature: &str) -> bool;
}
