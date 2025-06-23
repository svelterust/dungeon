//! Simplified game module that re-exports the main game functionality
//!
//! This module serves as the main entry point for the game logic,
//! but the actual implementation has been refactored into separate
//! modules for better organization and maintainability.

pub use crate::game_state::run_client_game;