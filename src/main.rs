use std::io::{self, BufRead};

use std::env; // args
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Incorrect arguments.\nUsage: ./client server_ip server_port\nExample: ./client 192.168.1.15 2300");
        std::process::exit(-1);
    }

    let dest_ip = &args[1];
    let dest_port = &args[2];

    let mut stream = TcpStream::connect(format!("{}:{}",dest_ip, dest_port))?;

    println!("Client started and connected successfuly. Press Ctrl-C to stop reading."); 

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let (name, conf) = bmOS_client::parse_intent(line.unwrap().as_bytes()); // Pass the line as an &[u8]
        println!("Recognized: {},{}", name, conf);
        
        let mut name_to_send = name.to_owned();
        name_to_send.push('\n'); // Token to signal the end of the intent to be sent

        if conf > 0.60 { // Only send it if the confidence is high
            stream.write(name_to_send.as_bytes())?;
            println!("Confidence is high enough, sent");
            stream.flush().unwrap(); // Wait until all the bytes are written to the connection, as TcpStream is buffered
        } 
    }

    Ok(())
}
