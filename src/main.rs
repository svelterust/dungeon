// Modules
mod game;

// Imports
use anyhow::Result;
use game::run_standalone_game;

#[macroquad::main("Dungeon")]
async fn main() -> Result<()> {
    run_standalone_game().await
}
