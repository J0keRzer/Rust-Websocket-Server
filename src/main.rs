/* socket server main file */

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

mod utils;


// handles http upgrade handle to initiate socket connection
// sends response to the request
//
// doesn't return enytinhg
fn websocket_handshake(mut stream: &TcpStream) {
    let mut request = [0; 1024];
    stream.read(&mut request).expect("Error reading request");

    // Look for Sec-WebSocket-Key header
    // If it isn't present, throw error
    let key_start = request
        .windows(19)
        .position(|window| window == b"Sec-WebSocket-Key: " )
        .expect("Invalid WebSocket request") + 19;
    let key_end = key_start + request[key_start..].iter().position(|&c| c == b'\r').expect("Invalid WebSocket request");

    // to_string() at the end to convert to an owned String
    let key = String::from_utf8_lossy(&request[key_start..key_end]).trim().to_string();

    // Response key is calculated from request's Sec-WebSocket-Key
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        utils::calculate_accept_key(&key)
    );

    stream.write(response.as_bytes()).expect("Error writing response");
}

// writes the given message to the stream
// mask header in the socket frame is set to 0(false)
fn send_websocket_message(mut stream: &TcpStream, message: &str) {
    let frame = vec![
        0b1000_0001u8,
        message.len() as u8,
    ];

    stream.write_all(&frame).expect("Error sending WebSocket frame");
    stream.write_all(message.as_bytes()).expect("Error sending WebSocket message");
}

// Recive and decode message that is being read from the stream
//
// Returns:
// message - decoded data recived from client
fn recive_websocket_message(mut stream: &TcpStream) -> Option<String> {
    // First 2 bytes are frame's header
    // The 2nd one contains information about payload length
    let mut header = [0;2];
    if stream.read_exact(&mut header).is_err() {
        return None; // Connection closed
    }

    // Length is determined by checking if given in header len is <126 | 126 | 127
    let payload_length = match header[1] & 0b0111_1111 {
        126 => {
            // Extended payload length using the next 2 bytes
            let mut extended_payload_length_bytes = [0; 2];
            if stream.read_exact(&mut extended_payload_length_bytes).is_err() {
                return None; // Connection closed
            }
            u16::from_be_bytes(extended_payload_length_bytes) as usize
        }
        127 => {
            // Extended payload length using the next 8 bytes
            let mut extended_payload_length_bytes = [0; 8];
            if stream.read_exact(&mut extended_payload_length_bytes).is_err() {
                return None; // Connection closed
            }
            u64::from_be_bytes(extended_payload_length_bytes) as usize
        }
        length => length as usize,
    };

    let mut masking_key = [0; 4];

    // Handle masking if the Mask bit is set
    if (header[1] & 0b1000_0000) != 0 {
        if stream.read_exact(&mut masking_key).is_err() {
            return None; // Connection closed
        }
    }

    // Read payload data
    let mut payload = vec![0; payload_length];
    if stream.read_exact(&mut payload).is_err() {
        return None; // Connection closed
    }

    // If mask is enabled, decode the data wtih previously read key
    if (header[1] & 0b1000_0000) != 0 {
        // Unmask the payload
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= masking_key[i % 4];
        }
    }

    // Convert message to readble utf8 format
    // If characters are unreadable no error is thrown
    let message = String::from_utf8_lossy(&payload).into_owned();
    Some(message)
}

// Infinity loop that continues handling incoming requests
// until the message recived from recive_websocket_message is None
fn handle_connection(stream: &mut TcpStream) {
    // initiate handshake for the first request from client
    websocket_handshake(&stream);

    let welcome_msg = "Welcome to the WebSocket server!";
    send_websocket_message(&stream, welcome_msg);

    // recive and print messages until there are none
    loop {
        let message = recive_websocket_message(&stream);
        match message {
            Some(msg) => {
                println!("Recived message: {}", msg);
            }
            None => {
                println!("Ending connection");
                break
            }
        }
    }
}

fn main() {
    // localhost on port 8000
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("WebSocket server listening on: ws://127.0.0.1:8000");

    // for each request 
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
