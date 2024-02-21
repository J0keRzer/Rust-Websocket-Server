use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

mod utils;


fn websocket_handshake(mut stream: &TcpStream) {
    let mut request = [0; 1024];
    stream.read(&mut request).expect("Error reading request");

    let key_start = request
        .windows(19)
        .position(|window| window == b"Sec-WebSocket-Key: " )
        .expect("Invalid WebSocket request") + 19;
    let key_end = key_start + request[key_start..].iter().position(|&c| c == b'\r').expect("Invalid WebSocket request");

    // to_string() at the end to convert to an owned String
    let key = String::from_utf8_lossy(&request[key_start..key_end]).trim().to_string();

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        utils::calculate_accept_key(&key)
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

    // Handle masking if the Mask bit is set
    let mut masking_key = [0; 4];

    if (header[1] & 0b1000_0000) != 0 {
        if stream.read_exact(&mut masking_key).is_err() {
            return None; // Connection closed
        }
    }

    let mut payload = vec![0; payload_length];
    if stream.read_exact(&mut payload).is_err() {
        return None; // Connection closed
    }

    if (header[1] & 0b1000_0000) != 0 {
        // Unmask the payload
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= masking_key[i % 4];
        }
    }

    // Debugging statements
    println!("Header: {:?}", header);
    println!("Payload Length: {}", payload_length);
    println!("Payload Masked: {}", String::from_utf8_lossy(&payload));



    let message = String::from_utf8_lossy(&payload).into_owned();
    Some(message)
}

fn handle_connection(stream: &mut TcpStream) {
    websocket_handshake(&stream);

    let welcome_msg = "Welcome to the WebSocket server!";
    send_websocket_message(&stream, welcome_msg);

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
