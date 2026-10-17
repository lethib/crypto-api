use serde_json::{Map, Value};

mod base64;
pub use self::base64::Base64Encryptor;

type Byte = u8;
// used to detect encrypted data from non-encrypted data but still valid base64 strings
const ENCRYPTION_PREFIX: &str = "enc:";

// Send and Sync trait are required to safely send data across threads
pub trait Encryptor: Send + Sync {
    fn encrypt(&self, data_to_encrypt: &[Byte]) -> String;
    fn decrypt(&self, data_to_decrypt: &str) -> Option<Vec<Byte>>;
}

pub fn encrypt_fields(encryptor: &dyn Encryptor, fields: Map<String, Value>) -> Map<String, Value> {
    fields
        .into_iter()
        .map(|(key, val)| (key, encrypt_value(encryptor, &val)))
        .collect()
}

fn encrypt_value(encryptor: &dyn Encryptor, value: &Value) -> Value {
    let encrypted_field = encryptor.encrypt(value.to_string().as_bytes());
    Value::String(format!("{ENCRYPTION_PREFIX}{}", encrypted_field))
}

pub fn decrypt_fields(encryptor: &dyn Encryptor, fields: Map<String, Value>) -> Map<String, Value> {
    fields
        .into_iter()
        .map(|(key, val)| (key, decrypt_value(encryptor, val)))
        .collect()
}

fn decrypt_value(encryptor: &dyn Encryptor, value: Value) -> Value {
    // if value is not a string, we return the original value
    let Value::String(text) = &value else {
        return value;
    };
    let Some(encrypted_data) = text.strip_prefix(ENCRYPTION_PREFIX) else {
        return value;
    };

    encryptor
        .decrypt(encrypted_data)
        .and_then(|decrypted_data| serde_json::from_slice(&decrypted_data).ok())
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::encryption::{base64::Base64Encryptor, decrypt_value, encrypt_value};

    fn base64_roundtrip(value: Value) {
        let base64_encryptor = Base64Encryptor {};
        let encrypted = encrypt_value(&base64_encryptor, &value);
        let decrypted = decrypt_value(&base64_encryptor, encrypted);

        assert_eq!(value, decrypted)
    }

    #[test]
    fn roundtrips_string() {
        base64_roundtrip(json!("John Doe"));
    }

    #[test]
    fn roundtrips_empty_string() {
        base64_roundtrip(json!(""));
    }

    #[test]
    fn roundtrips_unicode_string() {
        base64_roundtrip(json!("héhé 日本 🎮"));
    }

    #[test]
    fn roundtrips_integer() {
        base64_roundtrip(json!(30));
    }

    #[test]
    fn roundtrips_negative_float() {
        base64_roundtrip(json!(-1.5));
    }

    #[test]
    fn roundtrips_boolean() {
        base64_roundtrip(json!(true));
    }

    #[test]
    fn roundtrips_null() {
        base64_roundtrip(json!(null));
    }

    #[test]
    fn roundtrips_nested_object() {
        base64_roundtrip(json!({
            "email": "john@example.com",
            "phone": "123-456-7890",
            "address": { "city": "Paris", "zip": null }
        }));
    }

    #[test]
    fn roundtrips_array() {
        base64_roundtrip(json!([1, "two", { "three": [3.0] }]));
    }

    #[cfg(test)]
    mod when_data_is_encrypted {
        use serde_json::json;

        use crate::encryption::{Base64Encryptor, ENCRYPTION_PREFIX, encrypt_value};

        #[test]
        fn it_should_start_with_encryption_prefix() {
            let encrypted = encrypt_value(&Base64Encryptor, &json!("data_to_encrypt"));
            let text = encrypted.as_str().unwrap();

            assert!(text.starts_with(ENCRYPTION_PREFIX));
        }
    }

    mod when_not_encrypted_data_are_decrypted {
        use serde_json::json;

        use crate::encryption::{Base64Encryptor, decrypt_value};

        #[test]
        fn it_should_not_change() {
            let decrypted = decrypt_value(&Base64Encryptor, json!("1998-11-19"));
            assert_eq!(decrypted, "1998-11-19")
        }
    }

    mod when_base64_encrypted_data_are_decrypted {
        use serde_json::json;

        use crate::encryption::{Base64Encryptor, decrypt_value};

        #[test]
        fn it_should_unchange_invalid_data() {
            let decrypted = decrypt_value(&Base64Encryptor, json!("enc:!!!"));
            assert_eq!(decrypted, "enc:!!!")
        }

        #[test]
        fn it_should_unchange_invalid_json_data() {
            // aGVsbG8= -> hello, which is not a valid json data
            let decrypted = decrypt_value(&Base64Encryptor, json!("enc:aGVsbG8="));
            assert_eq!(decrypted, "enc:aGVsbG8=")
        }
    }

    mod golden_path {
        use serde_json::{Value, json};

        use crate::encryption::{Base64Encryptor, decrypt_fields, encrypt_fields};

        #[test]
        fn it_should_correctly_encrypt_and_decrypt_a_valid_json() {
            let Value::Object(fields) = json!({
              "name": "John Doe",
              "age": 30,
              "contact": {
                "email": "john@example.com",
                "phone": "123-456-7890"
              }
            }) else {
                unreachable!("json! with braces always builds an object")
            };

            let encrypted_fields = encrypt_fields(&Base64Encryptor, fields.clone());

            let Value::Object(expected) = json!({
                "name": "enc:IkpvaG4gRG9lIg==",
                "age": "enc:MzA=",
                "contact": "enc:eyJlbWFpbCI6ImpvaG5AZXhhbXBsZS5jb20iLCJwaG9uZSI6IjEyMy00NTYtNzg5MCJ9"
            }) else {
                unreachable!("json! with braces always builds an object")
            };

            assert_eq!(encrypted_fields, expected);

            let decrypted_fields = decrypt_fields(&Base64Encryptor, encrypted_fields);

            assert_eq!(decrypted_fields, fields);
        }
    }
}
