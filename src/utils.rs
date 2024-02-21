use sha1::{Digest, Sha1};
use base64::encode;


pub fn calculate_accept_key(key: &str) -> String {
    const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let combined_key = format!("{key}{GUID}");
    let key_bytes = combined_key.as_bytes();

    let sha1_hash = sha1::Sha1::digest(key_bytes);

    base64::encode(sha1_hash)
}