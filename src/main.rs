use std::net::{TcpListener, TcpStream};
use sha1::{Digest, Sha1};
use base64::encode;
use std::io::{Read, Write};


fn calculate_accept_key(key: &str) -> String {
    const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let combined_key = format!("{key}{GUID}");
    let key_bytes = combined_key.as_bytes();

    let sha1_hash = sha1::Sha1::digest(key_bytes);

    base64::encode(sha1_hash)
}

fn websocket_handshake(mut stream: &TcpStream) {
    let mut request = [0; 1024];
    stream.read(&mut request).expect("Error reading request");

    let key_start = request
        .windows(19)
        .position(|window| window == b"Sec-WebSocket-Key:" )
        .expect("Invalid WebSocket request") + 19;
    let key_end = key_start + request[key_start..].iter().position(|&c| c == b'\r').expect("Invalid WebSocket request");

    // to_string() at the end to convert to an owned String
    let key = String::from_utf8_lossy(&request[key_start..key_end]).trim().to_string();

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        calculate_accept_key(&key)
    );

    stream.write(response.as_bytes()).expect("Error writing response");
}

fn send_websocket_message(mut stream: &TcpStream, message: &str) {
    let frame = vec![
        0b1000_0001u8,
        message.len() as u8,
    ];

    stream.write_all(&frame).expect("Error sending WebSocket frame");
    stream.write_all(message.as_bytes()).expect("Error sending WebSocket message");
}

fn recive_websocket_message(mut stream: &TcpStream) -> Option<String> {
    let mut header = [0;2];
    if stream.read_exact(&mut header).is_err() {
        return None; // Connection closed
    }

    let payload_length = header[1] as usize & 0b0111_1111;
    let mut payload = vec![0; payload_length];
    if stream.read_exact(&mut payload).is_err() {
        return None; // Connection closed
    }

    let message = String::from_utf8_lossy(&payload).into_owned();
    Some(message)
}

fn handle_connection(stream: &mut TcpStream) {
    websocket_handshake(&stream);

    let welcome_msg = "Welcome to the WebSocket server!";
    send_websocket_message(&stream, welcome_msg);

    loop {
        let message = recive_websocket_message(&stream);
        match(message) {
            Some(msg) => {
                println!("Recived message: {}", msg);
            }
            None => break,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("WebSocket server listening on: ws://127.0.0.1:8000");


    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) =>  {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("Error accepting connection: {}", e);
            }
        }

        println!("Connection established!");
    }
}
