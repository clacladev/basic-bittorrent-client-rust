use serde_bencode;
use std::env;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let mut encoded_value_chars = encoded_value.chars();
    let encoded_value_first_char = encoded_value_chars.next().unwrap();

    if encoded_value_first_char.is_digit(10) {
        // If encoded_value starts with a digit, it's a string
        // Example: "5:hello" -> "hello"
        let decoded_value: String = serde_bencode::de::from_str(&encoded_value)
            .unwrap_or_else(|e| panic!("Failed to decode: {}. Error: {}", encoded_value, e));
        return serde_json::Value::String(decoded_value.to_string());
    } else if encoded_value_first_char.eq(&'i') {
        // It's an integer
        let decoded_value: i64 = serde_bencode::de::from_str(&encoded_value)
            .unwrap_or_else(|e| panic!("Failed to decode: {}. Error: {}", encoded_value, e));
        return serde_json::Value::Number(serde_json::Number::from(decoded_value));
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
