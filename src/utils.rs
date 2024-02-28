use sha1::{Digest, Sha1};
use base64;


// Calculates key that server needs for response by using server's GUID
// Used GUID is random, can be changed freely
//
// Takes:
// key - Sec-WebSocket-Key from client's request
//
// Returns:
// Server's response key for given client
pub fn calculate_accept_key(key: &str) -> String {
    // Random key
    const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    // Client's key combined with server's GUID
    let combined_key = format!("{key}{GUID}");
    let key_bytes = combined_key.as_bytes();

    let sha1_hash = Sha1::digest(key_bytes);

    // Every key needs to be base64 encoded
    base64::encode(sha1_hash)
}