// Modules
mod game;

// Imports
use argh::FromArgs;
use bincode;
use game::Payload;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(FromArgs)]
/// Dungeon multiplayer server
struct Args {
    #[argh(option, short = 'p', default = "8080")]
    /// port to listen on
    port: u16,
}

struct Server {
    clients: Arc<Mutex<HashMap<String, TcpStream>>>,
    client_counter: Arc<Mutex<u32>>,
}

impl Server {
    fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_counter: Arc::new(Mutex::new(0)),
        }
    }

    fn start(&self, address: &str) -> std::io::Result<()> {
        // Start server on address
        let listener = TcpListener::bind(address)?;
        println!("Server listening on {address}");

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
        let client_id = {
            let mut counter = counter.lock().unwrap();
            *counter += 1;
            format!("client_{}", *counter)
        };

        println!("New client connected: {}", client_id);

        let stream_clone = stream.try_clone().expect("Failed to clone stream");
        {
            let mut clients = clients.lock().unwrap();
            clients.insert(client_id.clone(), stream_clone);
        }

        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client {client_id} disconnected");
                    let mut clients = clients.lock().unwrap();
                    clients.remove(&client_id);
                    break;
                }
                Ok(n) => {
                    if n >= 4 {
                        let len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
                            as usize;
                        if n >= 4 + len {
                            if let Ok(_payload) =
                                bincode::deserialize::<Payload>(&buffer[4..4 + len])
                            {
                                Self::broadcast_to_others(&clients, &client_id, &buffer[..4 + len]);
                            }
                        }
                    }
                }
                Err(_) => {
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
        message: &[u8],
    ) {
        let mut clients = clients.lock().unwrap();
        let mut disconnected_clients = Vec::new();

        for (client_id, stream) in clients.iter_mut() {
            if client_id != sender_id {
                if stream.write_all(message).is_err() {
                    disconnected_clients.push(client_id.clone());
                }
            }
        }

        for client_id in disconnected_clients {
            clients.remove(&client_id);
        }
    }
}

fn main() -> std::io::Result<()> {
    // Start the server
    let Args { port } = argh::from_env::<Args>();
    let server = Server::new();
    server.start(&format!("0.0.0.0:{port}"))
}
