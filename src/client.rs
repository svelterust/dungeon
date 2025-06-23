// Modules
mod game;

// Miports
use anyhow::{Result, anyhow};
use argh::FromArgs;
use game::{Payload, run_client_game};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(FromArgs)]
/// Dungeon multiplayer client
struct Args {
    #[argh(option, short = 'a', default = "\"localhost:8080\".to_string()")]
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
        let mut read_stream = stream.try_clone()?;
        let write_stream = stream;
        println!("Connected to server at {}", address);

        // Handle incoming messages
        let incoming_handle = {
            let incoming = incoming.clone();
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match read_stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Server disconnected");
                            break;
                        }
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buffer[..n]);
                            if let Ok(payload) = serde_json::from_str::<Payload>(&data) {
                                if incoming.send(payload).is_err() {
                                    break;
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
                    if let Ok(json) = serde_json::to_string(&payload) {
                        if stream.write_all(json.as_bytes()).is_err() {
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
    NetworkClient::connect(&address, game_to_net_rx, net_to_game_tx)?;
    run_client_game(game_to_net_tx, net_to_game_rx, &address).await
}
