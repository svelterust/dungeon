//! Input handling system for player movement and actions

use crate::Payload;
use crate::constants::player;
use crate::entities::{Bullet, Player};
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
    ) -> bool {
        // Handle respawning
        if !local_player.is_alive {
            Self::handle_respawn(local_player, network_sender);
            return false;
        }

        // Handle shooting
        if is_key_pressed(KeyCode::Space) {
            Self::handle_shooting(local_player, bullets, network_sender);
        }

        // Move player and return if moved
        Self::handle_movement(local_player, network_sender)
    }

    /// Handle player respawn logic
    fn handle_respawn(local_player: &mut Player, network_sender: &Option<Sender<Payload>>) {
        let dt = get_frame_time();
        local_player.update_respawn(dt);

        if local_player.can_respawn() {
            local_player.respawn();

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
        let mut new_direction_x = local_player.direction_x;
        let mut new_direction_y = local_player.direction_y;

        // Handle movement keys
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            local_player.move_by(-speed, 0.0);
            new_direction_x = -1.0;
            new_direction_y = 0.0;
            moved = true;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            local_player.move_by(speed, 0.0);
            new_direction_x = 1.0;
            new_direction_y = 0.0;
            moved = true;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            local_player.move_by(0.0, -speed);
            new_direction_x = 0.0;
            new_direction_y = -1.0;
            moved = true;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            local_player.move_by(0.0, speed);
            new_direction_x = 0.0;
            new_direction_y = 1.0;
            moved = true;
        }

        // Update direction and send to network if changed
        if local_player.set_direction(new_direction_x, new_direction_y)
            && let Some(sender) = network_sender {
                let _ = sender.send(Payload::PlayerDirection(
                    local_player.id,
                    local_player.direction_x,
                    local_player.direction_y,
                ));
            }

        moved
    }

    /// Handle shooting input
    fn handle_shooting(
        local_player: &Player,
        bullets: &mut Vec<Bullet>,
        network_sender: &Option<Sender<Payload>>,
    ) {
        let bullet = Bullet::new(
            local_player.x,
            local_player.y,
            local_player.direction_x,
            local_player.direction_y,
            local_player.id,
        );
        bullets.push(bullet);

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
