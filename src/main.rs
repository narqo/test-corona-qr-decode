use std::{env, path};

use compress::zlib;
use serde::Deserialize;
use serde_cbor::value::Value;

fn main() {
    let path = match env::args_os().nth(1) {
        Some(path) => path::PathBuf::from(path),
        None => {
            panic!("Usage: cmd <path>");
        }
    };

    let img = image::open(path).unwrap();

    let decoder = bardecoder::default_decoder();
    let results = decoder.decode(&img);

    assert_eq!(1, results.len());

    if let Ok(line) = results.first().unwrap() {
        if let Some(input) = line.strip_prefix("HC1:") {
            decode(input);
        }
    }
}

#[derive(Debug, Deserialize)]
struct CWT {
    alg: Value,
    kid: Value,
    payload: Value,
    signature: Value,
}

fn decode(input: &str) {
    let raw_data = base45::decode(input).unwrap();
    let zlib_dec = zlib::Decoder::new(raw_data.as_slice());

    let cwt: CWT = serde_cbor::from_reader(zlib_dec).unwrap();

    if let Value::Bytes(payload) = cwt.payload {
        let value: Value = serde_cbor::from_slice(&payload).unwrap();
        print_value(&value, 0);
    }
}

fn print_value(value: &Value, level: usize) {
    let indent = "  ".repeat(level);
    match value {
        Value::Map(data) => {
            for (k, v) in data.iter() {
                match (k, v) {
                    (Value::Text(k), Value::Text(v)) => {
                        println!("{}{}: {}", indent, k, v)
                    }
                    (Value::Text(k), Value::Integer(v)) => {
                        println!("{}{}: {}", indent, k, v)
                    }
                    (Value::Integer(k), Value::Integer(v)) => {
                        println!("{}{}: {}", indent, k, v)
                    }
                    (Value::Integer(k), Value::Text(v)) => {
                        println!("{}{}: {}", indent, k, v)
                    }
                    (Value::Text(k), _) => {
                        println!("{}{}:", indent, k);
                        print_value(v, level + 1);
                    }
                    (Value::Integer(k), _) => {
                        println!("{}{}:", indent, k);
                        print_value(v, level + 1);
                    }
                    _ => {}
                }
            }
        }
        Value::Array(data) => {
            for (n, v) in data.iter().enumerate() {
                println!("{}{}:", indent, n);
                print_value(v, level + 1);
            }
        }
        Value::Text(text) => {
            println!("{}{}", indent, text)
        }
        Value::Integer(text) => {
            println!("{}{}", indent, text)
        }
        _ => {}
    }
}
