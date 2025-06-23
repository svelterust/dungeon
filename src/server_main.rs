// Modules
mod game;
mod server;

// Imports
use anyhow::Result;
use argh::FromArgs;
use server::Server;

#[derive(FromArgs)]
/// Dungeon multiplayer server
struct Args {
    #[argh(option, short = 'p', default = "8080")]
    /// port to listen on
    port: u16,
}

fn main() -> Result<()> {
    // Start server on port
    let Args { port } = argh::from_env::<Args>();
    println!("Starting server on port {port}");
    let server = Server::new();
    let address = format!("0.0.0.0:{port}");
    Ok(server.start(&address)?)
}
