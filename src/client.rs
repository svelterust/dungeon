// Modules
mod game;

// Miports
use anyhow::Result;
use argh::FromArgs;
use game::{Payload, run_client_game};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(FromArgs)]
/// Dungeon multiplayer client
struct Args {
    #[argh(option, short = 'a', default = "\"0.0.0.0:8080\".to_string()")]
    /// server address to connect to
    address: String,
}

struct NetworkClient {
    handle: thread::JoinHandle<()>,
}

impl NetworkClient {
    fn connect(
        address: &str,
        outgoing: Receiver<Payload>,
        incoming: Sender<Payload>,
    ) -> Result<Self, std::io::Error> {
        // Connect to server
        let stream = TcpStream::connect(address)?;
        stream.set_nodelay(true)?;
        let mut read_stream = stream.try_clone()?;
        let write_stream = stream;
        println!("Connected to server at {address}");

        // Handle incoming messages
        let incoming_handle = {
            let incoming = incoming.clone();
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                let mut message_buffer = Vec::new();
                loop {
                    match read_stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Server disconnected");
                            break;
                        }
                        Ok(n) => {
                            message_buffer.extend_from_slice(&buffer[..n]);
                            
                            // Process all complete messages
                            while message_buffer.len() >= 4 {
                                let len = u32::from_le_bytes([
                                    message_buffer[0], message_buffer[1], 
                                    message_buffer[2], message_buffer[3],
                                ]) as usize;
                                
                                if message_buffer.len() >= 4 + len {
                                    if let Ok(payload) = bincode::deserialize::<Payload>(&message_buffer[4..4 + len]) {
                                        if incoming.send(payload).is_err() {
                                            return;
                                        }
                                    }
                                    message_buffer.drain(..4 + len);
                                } else {
                                    break; // Wait for more data
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
        };

        // Handle outgoing messages
        let outgoing_handle = {
            thread::spawn(move || {
                let mut stream = write_stream;
                while let Ok(payload) = outgoing.recv() {
                    if let Ok(data) = bincode::serialize(&payload) {
                        let len = (data.len() as u32).to_le_bytes();
                        if stream.write_all(&len).is_err() || stream.write_all(&data).is_err() {
                            break;
                        }
                        if stream.flush().is_err() {
                            break;
                        }
                    }
                }
            })
        };

        let handle = thread::spawn(move || {
            let _ = incoming_handle.join();
            let _ = outgoing_handle.join();
        });
        Ok(NetworkClient { handle: handle })
    }
}

#[macroquad::main("Dungeon")]
async fn main() -> Result<()> {
    // Create channels for communication between game and network
    let (game_to_net_tx, game_to_net_rx) = mpsc::channel::<Payload>();
    let (net_to_game_tx, net_to_game_rx) = mpsc::channel::<Payload>();

    // Start network client and run game loop
    let Args { address } = argh::from_env::<Args>();
    println!("Connecting to server at {address}...");
    
    // Generate unique client ID from timestamp
    let player_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    
    NetworkClient::connect(&address, game_to_net_rx, net_to_game_tx)?;
    run_client_game(game_to_net_tx, net_to_game_rx, player_id).await
}
