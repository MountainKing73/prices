use std::collections::HashMap;
use std::io::BufReader;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("Listening on port 8080");
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        println!("Starting connection");
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    process_request(&mut stream);
                });
            }
            Err(err) => panic!("{}", err),
        }
    }
}

fn convert_number(num: &[u8]) -> i64 {
    let mut bin_string = "".to_string();
    for n in num {
        bin_string.push_str(&format!("{:08b}", n));
    }

    u64::from_str_radix(&bin_string, 2).unwrap() as i64
}

fn convert_response(num: i64) -> [u8; 4] {
    let bin_string = format!("{:032b}", num);

    let mut result: [u8; 4] = [0; 4];
    for i in 0..4 {
        result[i] = u8::from_str_radix(&bin_string[i * 8..((i * 8) + 8)], 2).unwrap();
    }

    result
}

fn process_request(stream: &mut TcpStream) {
    let mut entries: HashMap<u64, i64> = HashMap::new();
    let mut reader = BufReader::new(stream.try_clone().expect("stream clone failed"));

    let mut buffer = [0; 9];

    loop {
        let n = reader.read_exact(&mut buffer[..]);
        if n.is_err() {
            break;
        }

        println!("Received: {:?}", buffer);

        if buffer[0] == b'I' {
            let timestamp = convert_number(&buffer[1..5]) as u64;
            let price = convert_number(&buffer[5..]);
            entries.insert(timestamp, price);
        } else if buffer[0] == b'Q' {
            let timestamp1 = convert_number(&buffer[1..5]) as u64;
            let timestamp2 = convert_number(&buffer[5..]) as u64;

            let mut total = 0;
            let mut count = 0;
            for entry in &entries {
                if timestamp1 <= *entry.0 && timestamp2 >= *entry.0 {
                    total += entry.1;
                    count += 1;
                }
            }

            let mut result: i64 = 0;
            if count > 0 {
                result = total / count;
            }

            let _ = stream.write(&convert_response(result));
        } else {
            panic!("Invalid message type");
        }
    }
}
