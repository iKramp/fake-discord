use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

pub struct ServerNetworking {
   // channels: HashMap<u64, Vec<TcpStream>>,
    clients : Vec<TcpStream>,
}


impl ServerNetworking{
    pub const fn new() -> Self {
        Self{
            clients : Vec::new(),
        }
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) -> Result<(), Error> {
        println!("Incoming connection from: {}", stream.peer_addr()?);
        let mut buf = [0; 512];
        
        loop {
            let bytes_read = stream.read(&mut buf)?;
            if bytes_read == 0 {
                return Ok(());
            }
            println!("Received message");
            stream.write_all(buf.get(..bytes_read).unwrap())?;
            println!("Echoed");
        }
    }

    pub fn listen_for_client(&mut self) {
        //listen on port 8080
        let listener = TcpListener::bind("localhost:8080").unwrap();
        println!("Server listening on port 8080");

        // spawn a new thread for each connection
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {} ", stream.peer_addr().unwrap());
                    //self.clients.push(stream.try_clone().unwrap());
                    thread::spawn(move || {
                        self.handle_client(stream).unwrap_or_else(|error| eprintln!("{error:?}"));
                    });
                }
                Err(e) => {
                    println!("Error: {e}");
                }
            }
        }
        //close socket server
        println!("Stopping listening");
        drop(listener);
    }

}