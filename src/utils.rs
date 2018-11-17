use crypto::buffer::{ReadBuffer, WriteBuffer};
use failure::Fail;
use futures::future;
use futures::prelude::*;
use hyper;
use regex;
use sentry::integrations::failure::capture_error;

pub fn format_error<E: Fail>(error: &E) -> String {
    let mut result = String::new();
    let mut chain: Vec<&Fail> = Vec::new();
    let mut iter: Option<&Fail> = Some(error);
    while let Some(e) = iter {
        chain.push(e);
        iter = e.cause();
    }
    for err in chain.into_iter().rev() {
        result.push_str(&format!("{}\n", err));
    }
    if let Some(bt) = error.backtrace() {
        let regexp = regex::Regex::new("keystore_lib").unwrap();
        let bt = format!("{}", bt);
        let lines: Vec<&str> = bt.split("\n").skip(1).collect();
        if lines.len() > 0 {
            result.push_str("\nRelevant backtrace:\n");
        }
        lines.chunks(2).for_each(|chunk| {
            if let Some(line1) = chunk.get(0) {
                if regexp.is_match(line1) {
                    result.push_str(line1);
                    result.push_str("\n");
                    if let Some(line2) = chunk.get(1) {
                        result.push_str(line2);
                        result.push_str("\n");
                    }
                }
            }
        });
    }
    result
}

pub fn log_error<E: Fail>(error: &E) {
    error!("\n{}", format_error(error));
}

pub fn log_and_capture_error<E: Fail>(error: E) {
    log_error(&error);
    capture_error(&error.into());
}

pub fn log_warn<E: Fail>(error: &E) {
    warn!("\n{}", format_error(error));
}

// Reads body of request in Future format
pub fn read_body(body: hyper::Body) -> impl Future<Item = Vec<u8>, Error = hyper::Error> {
    body.fold(Vec::new(), |mut acc, chunk| {
        acc.extend_from_slice(&*chunk);
        future::ok::<_, hyper::Error>(acc)
    })
}

pub fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, crypto::symmetriccipher::SymmetricCipherError> {
    let mut encryptor = crypto::aes::cbc_encryptor(crypto::aes::KeySize::KeySize256, key, iv, crypto::blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));

        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            crypto::buffer::BufferResult::BufferUnderflow => break,
            crypto::buffer::BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, crypto::symmetriccipher::SymmetricCipherError> {
    let mut decryptor = crypto::aes::cbc_decryptor(crypto::aes::KeySize::KeySize256, key, iv, crypto::blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            crypto::buffer::BufferResult::BufferUnderflow => break,
            crypto::buffer::BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut res = String::new();
    for byte in bytes.iter() {
        res.push_str(&format!("{:02x}", byte));
    }
    res
}

pub fn decode_hex(hex_str: &str) -> Vec<u8> {
    hex_str
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let mut hex = String::new();
            hex.push(chunk[0].into());
            hex.push(chunk[1].into());
            u8::from_str_radix(&hex, 16).unwrap()
        }).collect()
}
