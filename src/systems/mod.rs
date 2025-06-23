//! Systems module for game logic organization
//!
//! This module contains specialized systems that handle different aspects of the game:
//! - Input system for handling player controls
//! - Collision system for detecting and resolving collisions
//! - Render system for drawing game elements and UI
//!
//! Each system follows the Single Responsibility Principle and can be easily tested
//! and maintained independently.

pub mod audio;
pub mod collision;
pub mod input;
pub mod network;
pub mod render;

pub use audio::AudioSystem;
pub use collision::CollisionSystem;
pub use input::InputSystem;
pub use network::NetworkSystem;
pub use render::RenderSystem;
