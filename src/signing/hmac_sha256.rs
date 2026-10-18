use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::signing::Signer;

pub struct HmacSha256Signer {
    key: String,
}

impl HmacSha256Signer {
    pub fn new(secret_key: String) -> Self {
        Self { key: secret_key }
    }

    fn mac(&self, message: &[u8]) -> Hmac<Sha256> {
        let mut hmac: Hmac<Sha256> = Hmac::new_from_slice(self.key.as_bytes()).unwrap();

        hmac.update(message);
        hmac
    }
}

impl Signer for HmacSha256Signer {
    fn sign(&self, message: &[u8]) -> String {
        let hmac = self.mac(message);

        hex::encode(hmac.finalize().into_bytes())
    }

    fn verify(&self, message: &[u8], signature: &str) -> bool {
        let Ok(decoded_signature) = hex::decode(signature) else {
            return false;
        };

        self.mac(message).verify_slice(&decoded_signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::HmacSha256Signer;
    use crate::signing::Signer;

    const SECRET: &str = "a-secret-key-for-tests-only-0123456789";
    const OTHER_SECRET: &str = "another-secret-key-0123456789-abcdef";

    fn signer() -> HmacSha256Signer {
        HmacSha256Signer::new(SECRET.to_string())
    }

    fn other_signer() -> HmacSha256Signer {
        HmacSha256Signer::new(OTHER_SECRET.to_string())
    }

    // Official test vectors from RFC 4231: they prove this is the real HMAC-SHA256.
    mod when_signing_a_known_vector {
        use super::*;

        #[test]
        fn it_should_match_rfc_4231_test_case_1() {
            let signer = HmacSha256Signer::new("\x0b".repeat(20));

            assert_eq!(
                signer.sign("Hi There".as_bytes()),
                "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
            );
        }

        #[test]
        fn it_should_match_rfc_4231_test_case_2() {
            let signer = HmacSha256Signer::new("Jefe".to_string());

            assert_eq!(
                signer.sign("what do ya want for nothing?".as_bytes()),
                "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
            );
        }
    }

    mod when_signing {
        use super::*;

        #[test]
        fn it_should_be_deterministic() {
            assert_eq!(
                signer().sign("message".as_bytes()),
                signer().sign("message".as_bytes())
            );
        }

        #[test]
        fn it_should_change_when_the_message_changes() {
            assert_ne!(
                signer().sign("message".as_bytes()),
                signer().sign("messagf".as_bytes())
            );
        }

        #[test]
        fn it_should_change_when_the_key_changes() {
            assert_ne!(
                signer().sign("message".as_bytes()),
                other_signer().sign("message".as_bytes())
            );
        }

        #[test]
        fn it_should_accept_an_empty_message() {
            let signature = signer().sign("".as_bytes());

            assert!(signer().verify("".as_bytes(), &signature));
        }
    }

    mod when_verifying_a_valid_signature {
        use super::*;

        #[test]
        fn it_should_return_true() {
            let signature = signer().sign("message".as_bytes());

            assert!(signer().verify("message".as_bytes(), &signature));
        }
    }

    mod when_verifying_a_tampered_input {
        use super::*;

        #[test]
        fn it_should_reject_a_modified_message() {
            let signature = signer().sign("message".as_bytes());

            assert!(!signer().verify("messagf".as_bytes(), &signature));
        }

        #[test]
        fn it_should_reject_a_modified_signature() {
            let mut signature = signer().sign("message".as_bytes());
            let first = if signature.starts_with('0') { "1" } else { "0" };
            signature.replace_range(0..1, first);

            assert!(!signer().verify("message".as_bytes(), &signature));
        }

        #[test]
        fn it_should_reject_a_signature_made_with_another_key() {
            let signature = other_signer().sign("message".as_bytes());

            assert!(!signer().verify("message".as_bytes(), &signature));
        }
    }
}
