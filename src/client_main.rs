use anyhow::Result;
use argh::FromArgs;
use std::sync::mpsc;

mod game;
mod client;

use game::{run_client_game, Payload};
use client::NetworkClient;

#[derive(FromArgs)]
/// Dungeon multiplayer client
struct Args {
    #[argh(option, short = 'a', default = "\"localhost:8080\".to_string()")]
    /// server address to connect to
    address: String,
}

#[macroquad::main("Dungeon Client")]
async fn main() -> Result<()> {
    let args: Args = argh::from_env();
    
    println!("Connecting to server at {}", args.address);

    // Create channels for communication between game and network
    let (game_to_net_tx, game_to_net_rx) = mpsc::channel::<Payload>();
    let (net_to_game_tx, net_to_game_rx) = mpsc::channel::<Payload>();

    // Start network client in background
    let _network_client = NetworkClient::connect(&args.address, game_to_net_rx, net_to_game_tx)
        .map_err(|e| anyhow::anyhow!("Failed to connect to server: {}", e))?;

    // Run game loop with network channels
    run_client_game(game_to_net_tx, net_to_game_rx, &args.address).await
}