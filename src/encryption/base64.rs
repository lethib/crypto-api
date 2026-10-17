use base64::{Engine, engine::general_purpose::STANDARD};

use super::{Byte, Encryptor};

pub struct Base64Encryptor;

impl Encryptor for Base64Encryptor {
    fn encrypt(&self, data_to_encrypt: &[Byte]) -> String {
        STANDARD.encode(data_to_encrypt)
    }

    fn decrypt(&self, data_to_decrypt: &str) -> Option<Vec<Byte>> {
        STANDARD.decode(data_to_decrypt).ok()
    }
}
