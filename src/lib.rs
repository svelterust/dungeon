//! Dungeon multiplayer game library
//!
//! This library provides the core game functionality for a multiplayer dungeon game
//! with boss fights, PvP combat, and real-time networking.

use serde::{Deserialize, Serialize};

// Core modules
pub mod constants;
pub mod entities;
pub mod systems;
pub mod game_state;
pub mod game;

// Re-export commonly used items
pub use entities::{Player, Boss, Bullet, AreaAttack, DamageIndicator};
pub use game_state::{GameState, run_client_game};
pub use game::run_client_game as run_game;

/// Network payload messages for multiplayer communication
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Move(u32, f32, f32), // player_id, x, y
    Join(u32),
    Leave(u32),
    Shoot(u32, f32, f32, f32, f32), // player_id, x, y, direction_x, direction_y
    BossShoot(f32, f32, f32, f32),  // x, y, direction_x, direction_y
    PlayerHit(u32, u32, u32),       // player_id, new_health, damage_amount
    BossHit(u32),                   // new_boss_health
    BossSpawn(f32, f32),            // x, y
    BossDead,
    BossMultiShoot(f32, f32, Vec<(f32, f32)>), // x, y, directions
    BossDash(f32, f32),                        // target_x, target_y
    BossAreaAttack(f32, f32),                  // center_x, center_y
    BossShield(bool),                          // shield_active
    PlayerRespawn(u32, f32, f32),              // player_id, x, y
    PlayerDirection(u32, f32, f32),            // player_id, direction_x, direction_y
    PlayerKill(u32, u32),                      // killer_id, victim_id
}
