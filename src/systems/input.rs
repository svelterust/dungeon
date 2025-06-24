//! Input handling system for player movement and actions

use crate::Payload;
use crate::constants::player;
use crate::entities::{Bullet, Player};
use crate::systems::AudioSystem;
use macroquad::prelude::*;
use std::sync::mpsc::Sender;

/// Handles all input processing for the local player
pub struct InputSystem;

impl InputSystem {
    /// Update player input and return whether player moved
    pub fn update_player_input(
        local_player: &mut Player,
        bullets: &mut Vec<Bullet>,
        network_sender: &Option<Sender<Payload>>,
        audio_system: &Option<AudioSystem>,
    ) -> bool {
        // Handle respawning
        if !local_player.is_alive {
            Self::handle_respawn(local_player, network_sender, audio_system);
            return false;
        }

        // Update player direction to face mouse cursor
        Self::update_mouse_direction(local_player, network_sender);

        // Handle shooting
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
            Self::handle_shooting(local_player, bullets, network_sender, audio_system);
        }

        // Move player and return if moved
        Self::handle_movement(local_player, network_sender)
    }

    /// Handle player respawn logic
    fn handle_respawn(local_player: &mut Player, network_sender: &Option<Sender<Payload>>, audio_system: &Option<AudioSystem>) {
        let dt = get_frame_time();
        local_player.update_respawn(dt);

        if local_player.can_respawn() {
            local_player.respawn();

            // Play respawn sound
            if let Some(audio) = audio_system {
                audio.play_respawn();
            }

            if let Some(sender) = network_sender {
                let _ = sender.send(Payload::PlayerRespawn(
                    local_player.id,
                    local_player.x,
                    local_player.y,
                ));
            }
        }
    }

    /// Handle player movement input
    fn handle_movement(
        local_player: &mut Player,
        network_sender: &Option<Sender<Payload>>,
    ) -> bool {
        let speed = player::SPEED * get_frame_time();
        let mut moved = false;

        // Handle movement keys (just movement, not direction)
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            local_player.move_by(-speed, 0.0);
            moved = true;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            local_player.move_by(speed, 0.0);
            moved = true;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            local_player.move_by(0.0, -speed);
            moved = true;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            local_player.move_by(0.0, speed);
            moved = true;
        }

        moved
    }

    /// Update player direction to face mouse cursor
    fn update_mouse_direction(
        local_player: &mut Player,
        network_sender: &Option<Sender<Payload>>,
    ) {
        let mouse_pos = mouse_position();
        let dx = mouse_pos.0 - local_player.x;
        let dy = mouse_pos.1 - local_player.y;
        
        // Calculate normalized direction vector
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 0.0 {
            let new_direction_x = dx / distance;
            let new_direction_y = dy / distance;
            
            // Update direction and send to network if changed
            if local_player.set_direction(new_direction_x, new_direction_y)
                && let Some(sender) = network_sender {
                    let _ = sender.send(Payload::PlayerDirection(
                        local_player.id,
                        local_player.direction_x,
                        local_player.direction_y,
                    ));
                }
        }
    }

    /// Handle shooting input
    fn handle_shooting(
        local_player: &Player,
        bullets: &mut Vec<Bullet>,
        network_sender: &Option<Sender<Payload>>,
        audio_system: &Option<AudioSystem>,
    ) {
        let bullet = Bullet::new(
            local_player.x,
            local_player.y,
            local_player.direction_x,
            local_player.direction_y,
            local_player.id,
        );
        bullets.push(bullet);

        // Play shooting sound
        if let Some(audio) = audio_system {
            audio.play_player_shoot();
        }

        // Send bullet to network
        if let Some(sender) = network_sender {
            let _ = sender.send(Payload::Shoot(
                local_player.id,
                local_player.x,
                local_player.y,
                local_player.direction_x,
                local_player.direction_y,
            ));
        }
    }

    /// Check if quit key is pressed
    pub fn should_quit() -> bool {
        is_key_pressed(KeyCode::Escape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_system_creation() {
        // InputSystem is a unit struct, so just test that it exists
        let _system = InputSystem;
    }
}
