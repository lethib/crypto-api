pub mod canonicalize;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> String;
    fn verify(&self, message: &[u8], signature: &str) -> bool;
}
