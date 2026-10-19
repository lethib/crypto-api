use serde_json::{Map, Value};

pub fn canonicalize(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.to_string(),
        Value::Array(val) => canonicalize_array(val),
        Value::Object(val) => canonicalize_object(val),
    }
}

fn canonicalize_array(arr: &[Value]) -> String {
    let members: Vec<String> = arr.iter().map(canonicalize).collect();
    format!("[{}]", members.join(","))
}

fn canonicalize_object(obj: &Map<String, Value>) -> String {
    let mut entries: Vec<(&String, &Value)> = obj.iter().collect();

    entries.sort_by_key(|k| k.0);

    let members: Vec<String> = entries
        .iter()
        .map(|(key, val)| {
            format!(
                "{}:{}",
                serde_json::to_string(key).expect("key is a string, can't fail"),
                canonicalize(val)
            )
        })
        .collect();

    format!("{{{}}}", members.join(","))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::canonicalize;

    mod when_value_is_a_leaf {
        use super::*;

        #[test]
        fn it_should_serialize_null() {
            assert_eq!(canonicalize(&json!(null)), "null");
        }

        #[test]
        fn it_should_serialize_booleans() {
            assert_eq!(canonicalize(&json!(true)), "true");
            assert_eq!(canonicalize(&json!(false)), "false");
        }

        #[test]
        fn it_should_serialize_numbers() {
            assert_eq!(canonicalize(&json!(30)), "30");
            assert_eq!(canonicalize(&json!(-1.5)), "-1.5");
        }

        #[test]
        fn it_should_quote_strings() {
            assert_eq!(canonicalize(&json!("John")), r#""John""#);
        }

        #[test]
        fn it_should_escape_special_characters_in_strings() {
            let original = json!("say \"hi\"\nnew line \\ backslash");
            let canonical = canonicalize(&original);

            // the output must be valid JSON that parses back to the same value
            let reparsed: serde_json::Value = serde_json::from_str(&canonical).unwrap();
            assert_eq!(reparsed, original);
        }

        #[test]
        fn it_should_keep_unicode_strings_intact() {
            let original = json!("héhé 日本 🎮");
            let reparsed: serde_json::Value =
                serde_json::from_str(&canonicalize(&original)).unwrap();
            assert_eq!(reparsed, original);
        }
    }

    mod when_value_is_an_array {
        use super::*;

        #[test]
        fn it_should_separate_items_with_commas_and_wrap_in_brackets() {
            assert_eq!(canonicalize(&json!([1, 2])), "[1,2]");
        }

        #[test]
        fn it_should_serialize_an_empty_array() {
            assert_eq!(canonicalize(&json!([])), "[]");
        }

        #[test]
        fn it_should_preserve_items_order() {
            assert_ne!(canonicalize(&json!([1, 2])), canonicalize(&json!([2, 1])));
        }

        #[test]
        fn it_should_sort_keys_of_objects_inside_arrays() {
            let first = json!([{ "b": 1, "a": 2 }, { "d": 3, "c": 4 }]);
            let second = json!([{ "a": 2, "b": 1 }, { "c": 4, "d": 3 }]);

            assert_eq!(canonicalize(&first), canonicalize(&second));
            assert_eq!(canonicalize(&first), r#"[{"a":2,"b":1},{"c":4,"d":3}]"#);
        }
    }

    mod when_value_is_an_object {
        use super::*;

        #[test]
        fn it_should_sort_keys() {
            assert_eq!(canonicalize(&json!({ "b": 1, "a": 2 })), r#"{"a":2,"b":1}"#);
        }

        #[test]
        fn it_should_serialize_an_empty_object() {
            assert_eq!(canonicalize(&json!({})), "{}");
        }

        #[test]
        fn it_should_not_depend_on_keys_order() {
            let first = json!({ "message": "Hello World", "timestamp": 1616161616 });
            let second = json!({ "timestamp": 1616161616, "message": "Hello World" });

            assert_eq!(canonicalize(&first), canonicalize(&second));
        }

        #[test]
        fn it_should_sort_keys_of_nested_objects() {
            let first = json!({ "b": { "z": 1, "y": 2 }, "a": [true, { "d": null, "c": 0 }] });
            let second = json!({ "a": [true, { "c": 0, "d": null }], "b": { "y": 2, "z": 1 } });

            assert_eq!(canonicalize(&first), canonicalize(&second));
            assert_eq!(
                canonicalize(&first),
                r#"{"a":[true,{"c":0,"d":null}],"b":{"y":2,"z":1}}"#
            );
        }

        #[test]
        fn it_should_escape_keys() {
            let original = json!({ "we\"ird\nkey": 1 });
            let canonical = canonicalize(&original);

            let reparsed: serde_json::Value = serde_json::from_str(&canonical).unwrap();
            assert_eq!(reparsed, original);
        }
    }

    // Two different values must never produce the same canonical form,
    // otherwise a tampered payload could reuse another payload's signature.
    mod when_values_are_different {
        use super::*;

        #[test]
        fn it_should_distinguish_a_string_from_a_number() {
            assert_ne!(canonicalize(&json!("1")), canonicalize(&json!(1)));
        }

        #[test]
        fn it_should_distinguish_a_string_from_a_boolean_and_null() {
            assert_ne!(canonicalize(&json!("true")), canonicalize(&json!(true)));
            assert_ne!(canonicalize(&json!("null")), canonicalize(&json!(null)));
        }

        #[test]
        fn it_should_distinguish_different_keys() {
            assert_ne!(
                canonicalize(&json!({ "a": 1 })),
                canonicalize(&json!({ "b": 1 }))
            );
        }

        #[test]
        fn it_should_distinguish_an_integer_from_a_float() {
            assert_ne!(canonicalize(&json!(1)), canonicalize(&json!(1.0)));
        }
    }
}
