use crate::game::Payload;
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Server {
    clients: Arc<Mutex<HashMap<String, TcpStream>>>,
    client_counter: Arc<Mutex<u32>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn start(&self, address: &str) -> std::io::Result<()> {
        let listener = TcpListener::bind(address)?;
        println!("Server listening on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);
                    let counter = Arc::clone(&self.client_counter);
                    thread::spawn(move || {
                        Self::handle_client(stream, clients, counter);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        Ok(())
    }

    fn handle_client(
        mut stream: TcpStream,
        clients: Arc<Mutex<HashMap<String, TcpStream>>>,
        counter: Arc<Mutex<u32>>,
    ) {
        // Generate client ID
        let client_id = {
            let mut counter = counter.lock().unwrap();
            *counter += 1;
            format!("client_{}", *counter)
        };

        println!("New client connected: {}", client_id);

        // Clone stream for storing in clients map
        let stream_clone = stream.try_clone().expect("Failed to clone stream");

        // Add client to the map
        {
            let mut clients = clients.lock().unwrap();
            clients.insert(client_id.clone(), stream_clone);
        }

        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // Client disconnected
                    println!("Client {} disconnected", client_id);
                    let mut clients = clients.lock().unwrap();
                    clients.remove(&client_id);
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    println!("Received from {}: {}", client_id, data);

                    // Try to parse as JSON payload
                    if let Ok(payload) = serde_json::from_str::<Payload>(&data) {
                        println!("Parsed payload: {:?}", payload);

                        // Broadcast to all other clients
                        Self::broadcast_to_others(&clients, &client_id, &data);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from client {}: {}", client_id, e);
                    let mut clients = clients.lock().unwrap();
                    clients.remove(&client_id);
                    break;
                }
            }
        }
    }

    fn broadcast_to_others(
        clients: &Arc<Mutex<HashMap<String, TcpStream>>>,
        sender_id: &str,
        message: &str,
    ) {
        let mut clients = clients.lock().unwrap();
        let mut disconnected_clients = Vec::new();

        for (client_id, stream) in clients.iter_mut() {
            if client_id != sender_id {
                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Failed to send to {}: {}", client_id, e);
                    disconnected_clients.push(client_id.clone());
                }
            }
        }

        // Remove disconnected clients
        for client_id in disconnected_clients {
            clients.remove(&client_id);
        }
    }
}
