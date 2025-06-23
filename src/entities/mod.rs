//! Game entities module
//! 
//! This module contains all the game entities like Player, Boss, Bullet, and visual effects.
//! Each entity is responsible for its own state management, behavior, and rendering.

pub mod player;
pub mod boss;
pub mod bullet;
pub mod effects;

pub use player::Player;
pub use boss::Boss;
pub use bullet::Bullet;
pub use effects::{AreaAttack, DamageIndicator};