use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::game::Payload;

pub struct NetworkClient {
    _handle: thread::JoinHandle<()>,
}

impl NetworkClient {
    pub fn connect(
        address: &str,
        outgoing: Receiver<Payload>,
        incoming: Sender<Payload>,
    ) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(address)?;
        println!("Connected to server at {}", address);

        let read_stream = stream.try_clone()?;
        let write_stream = stream;

        // Spawn thread to handle incoming messages
        let incoming_handle = {
            let incoming = incoming.clone();
            thread::spawn(move || {
                Self::handle_incoming(read_stream, incoming);
            })
        };

        // Spawn thread to handle outgoing messages
        let outgoing_handle = {
            thread::spawn(move || {
                Self::handle_outgoing(write_stream, outgoing);
            })
        };

        // Combine handles (in a real implementation you might want to track both)
        let handle = thread::spawn(move || {
            let _ = incoming_handle.join();
            let _ = outgoing_handle.join();
        });

        Ok(NetworkClient {
            _handle: handle,
        })
    }

    fn handle_incoming(mut stream: TcpStream, sender: Sender<Payload>) {
        let mut buffer = [0; 1024];
        
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Server disconnected");
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    
                    if let Ok(payload) = serde_json::from_str::<Payload>(&data) {
                        if sender.send(payload).is_err() {
                            println!("Game loop disconnected");
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_outgoing(mut stream: TcpStream, receiver: Receiver<Payload>) {
        while let Ok(payload) = receiver.recv() {
            match serde_json::to_string(&payload) {
                Ok(json) => {
                    if let Err(e) = stream.write_all(json.as_bytes()) {
                        eprintln!("Failed to send to server: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize payload: {}", e);
                }
            }
        }
        println!("Outgoing network thread ended");
    }
}