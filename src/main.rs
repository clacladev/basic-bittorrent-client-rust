use serde_bencode;
use std::env;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value)?;
    convert_bencode_value_to_json_value(value)
}

fn convert_bencode_value_to_json_value(
    value: serde_bencode::value::Value,
) -> anyhow::Result<serde_json::Value> {
    match value {
        serde_bencode::value::Value::Bytes(bytes) => {
            let string = String::from_utf8(bytes)?;
            Ok(serde_json::Value::String(string))
        }
        serde_bencode::value::Value::Int(int) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(int)))
        }
        serde_bencode::value::Value::List(values) => {
            let array = values
                .into_iter()
                .map(|item| convert_bencode_value_to_json_value(item))
                .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;
            Ok(serde_json::Value::Array(array))
        }
        serde_bencode::value::Value::Dict(hash_map) => {
            let mut map = serde_json::Map::new();
            for (key_bytes, json_value) in hash_map {
                let key = String::from_utf8(key_bytes)
                    .map_err(|_| anyhow::anyhow!("Failed to convert key to string"))?;
                let value = convert_bencode_value_to_json_value(json_value)?;
                map.insert(key, value);
            }
            Ok(serde_json::Value::Object(map))
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value)?;
        println!("{}", decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }

    Ok(())
}
