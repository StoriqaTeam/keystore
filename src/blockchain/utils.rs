use super::error::*;
use prelude::*;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut res = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter() {
        res.push_str(&format!("{:02x}", byte));
    }
    res
}

pub fn hex_to_bytes(hex: String) -> Result<Vec<u8>, Error> {
    let chars: Vec<char> = hex.clone().chars().collect();
    chars
        .chunks(2)
        .map(|chunk| {
            if chunk.len() < 2 {
                let error = ValidationError::MalformedHexString { value: hex.clone() };
                return Err(ErrorKind::Validation(error).into());
            }
            let string = format!("{}{}", chunk[0], chunk[1]);
            u8::from_str_radix(&string, 16).map_err({
                let error = ValidationError::MalformedHexString { value: hex.clone() };
                ectx!(ErrorKind::Validation(error))
            })
        })
        .collect()
}
