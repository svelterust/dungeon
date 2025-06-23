//! Main game state manager that coordinates all systems and entities

use crate::Payload;
use crate::constants::{multi_shot, network};
use crate::entities::{AreaAttack, Boss, Bullet, DamageIndicator, Player};
use crate::systems::{CollisionSystem, InputSystem, NetworkSystem, RenderSystem};
use anyhow::Result;
use macroquad::prelude::*;
use std::sync::mpsc::{Receiver, Sender};

/// Main game state that manages all entities and systems
pub struct GameState {
    // Entities
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub boss: Boss,
    pub area_attacks: Vec<AreaAttack>,
    pub damage_indicators: Vec<DamageIndicator>,

    // Systems
    collision_system: CollisionSystem,

    // Network
    network_sender: Option<Sender<Payload>>,
}

impl GameState {
    /// Create a new game state with the given player ID
    pub fn new(player_id: u32) -> Self {
        let local_player = Player::new_at_center(player_id);

        Self {
            local_player,
            remote_players: Vec::new(),
            bullets: Vec::new(),
            boss: Boss::new(),
            area_attacks: Vec::new(),
            damage_indicators: Vec::new(),
            collision_system: CollisionSystem::new(),
            network_sender: None,
        }
    }

    /// Set the network sender for multiplayer communication
    pub fn set_network_sender(&mut self, sender: Sender<Payload>) {
        self.network_sender = Some(sender);
    }

    /// Update game input and return whether the player moved
    pub fn update_input(&mut self) -> bool {
        InputSystem::update_player_input(
            &mut self.local_player,
            &mut self.bullets,
            &self.network_sender,
        )
    }

    /// Update all game entities
    pub fn update_entities(&mut self) {
        let dt = get_frame_time();

        // Update bullets
        self.bullets.retain_mut(|bullet| !bullet.update(dt));

        // Update area attacks
        self.area_attacks.retain_mut(|attack| !attack.update(dt));

        // Update damage indicators
        self.damage_indicators
            .retain_mut(|indicator| !indicator.update(dt));

        // Update remote player respawn timers
        for player in &mut self.remote_players {
            player.update_respawn(dt);
        }

        // Update boss
        self.update_boss();

        // Check collisions
        self.collision_system.check_bullet_collisions(
            &mut self.bullets,
            &mut self.local_player,
            &mut self.remote_players,
            &mut self.boss,
            &mut self.damage_indicators,
            &self.network_sender,
        );
    }

    /// Update boss AI and abilities
    fn update_boss(&mut self) {
        // Collect all alive players for boss AI
        let mut all_players = Vec::new();
        if self.local_player.is_alive {
            all_players.push(self.local_player.clone());
        }
        all_players.extend(self.remote_players.iter().filter(|p| p.is_alive).cloned());

        self.boss.update(get_frame_time(), &all_players);

        if all_players.is_empty() {
            return; // No players to target
        }

        // Handle boss abilities
        self.handle_boss_powers(&all_players);
        self.handle_boss_dash(&all_players);
        self.handle_boss_shooting(&all_players);
        self.handle_boss_respawn();
    }

    /// Handle boss power abilities
    fn handle_boss_powers(&mut self, players: &[Player]) {
        if self.boss.should_use_power() {
            let power_type = rand::gen_range(0, 3);
            match power_type {
                0 => self.execute_boss_multi_shot(players),
                1 => self.execute_boss_area_attack(players),
                2 => self.execute_boss_shield(),
                _ => {}
            }
            self.boss.reset_power_timer();
        }
    }

    /// Execute boss multi-shot ability
    fn execute_boss_multi_shot(&mut self, players: &[Player]) {
        if let Some(target_player) = self.find_nearest_player_to_boss(players) {
            let mut directions = Vec::new();

            // Create spread pattern bullets
            for i in multi_shot::ANGLE_RANGE {
                let angle_offset = i as f32 * multi_shot::SPREAD_ANGLE;
                let base_dx = target_player.x - self.boss.x;
                let base_dy = target_player.y - self.boss.y;
                let base_distance = (base_dx * base_dx + base_dy * base_dy).sqrt();

                if base_distance > 0.0 {
                    let base_angle = base_dy.atan2(base_dx);
                    let new_angle = base_angle + angle_offset;
                    let direction_x = new_angle.cos();
                    let direction_y = new_angle.sin();

                    directions.push((direction_x, direction_y));

                    let bullet =
                        Bullet::new_boss_bullet(self.boss.x, self.boss.y, direction_x, direction_y);
                    self.bullets.push(bullet);
                }
            }

            // Send multi-shot to network
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::BossMultiShoot(
                    self.boss.x,
                    self.boss.y,
                    directions,
                ));
            }
        }
    }

    /// Execute boss area attack ability
    fn execute_boss_area_attack(&mut self, players: &[Player]) {
        if let Some(target_player) = self.find_nearest_player_to_boss(players) {
            let area_center_x = target_player.x;
            let area_center_y = target_player.y;

            // Check if local player is affected
            let area_attack = AreaAttack::new(area_center_x, area_center_y);
            if self.local_player.is_alive
                && area_attack.affects_point(self.local_player.x, self.local_player.y)
            {
                let damage = area_attack.damage();
                self.local_player.take_damage(damage);

                // Add damage indicator
                self.damage_indicators.push(DamageIndicator::new(
                    self.local_player.x,
                    self.local_player.y,
                    damage,
                    false,
                ));

                if let Some(sender) = &self.network_sender {
                    let _ = sender.send(Payload::PlayerHit(
                        self.local_player.id,
                        self.local_player.health,
                    ));
                }
            }

            // Add visual effect
            self.area_attacks.push(area_attack);

            // Send area attack to network
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::BossAreaAttack(area_center_x, area_center_y));
            }
        }
    }

    /// Execute boss shield ability
    fn execute_boss_shield(&mut self) {
        self.boss.activate_shield();
        if let Some(sender) = &self.network_sender {
            let _ = sender.send(Payload::BossShield(true));
        }
    }

    /// Handle boss dash ability
    fn handle_boss_dash(&mut self, players: &[Player]) {
        if self.boss.should_dash() {
            if let Some(target_player) = self.find_nearest_player_to_boss(players) {
                self.boss.start_dash(target_player.x, target_player.y);
                if let Some(sender) = &self.network_sender {
                    let _ = sender.send(Payload::BossDash(target_player.x, target_player.y));
                }
            }
        }
    }

    /// Handle boss shooting
    fn handle_boss_shooting(&mut self, players: &[Player]) {
        if self.boss.should_shoot() && !self.boss.is_dashing {
            if let Some(target_player) = self.find_nearest_player_to_boss(players) {
                let dx = target_player.x - self.boss.x;
                let dy = target_player.y - self.boss.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 0.0 {
                    let direction_x = dx / distance;
                    let direction_y = dy / distance;

                    let bullet =
                        Bullet::new_boss_bullet(self.boss.x, self.boss.y, direction_x, direction_y);
                    self.bullets.push(bullet);

                    // Send boss shoot to network
                    if let Some(sender) = &self.network_sender {
                        let _ = sender.send(Payload::BossShoot(
                            self.boss.x,
                            self.boss.y,
                            direction_x,
                            direction_y,
                        ));
                    }
                }
            }
            self.boss.reset_shoot_timer();
        }
    }

    /// Handle boss respawn
    fn handle_boss_respawn(&mut self) {
        if self.boss.should_respawn() {
            self.boss.respawn();
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::BossSpawn(self.boss.x, self.boss.y));
            }
        }
    }

    /// Find the nearest player to the boss
    fn find_nearest_player_to_boss(&self, players: &[Player]) -> Option<Player> {
        players
            .iter()
            .filter(|p| p.is_alive)
            .min_by_key(|p| {
                let dx = p.x - self.boss.x;
                let dy = p.y - self.boss.y;
                (dx * dx + dy * dy) as i32
            })
            .cloned()
    }

    /// Process network messages
    pub fn process_network_messages(&mut self, network_receiver: &Receiver<Payload>) {
        let mut processed = 0;
        while let Ok(payload) = network_receiver.try_recv() {
            NetworkSystem::handle_message(
                &payload,
                &mut self.local_player,
                &mut self.remote_players,
                &mut self.boss,
                &mut self.bullets,
                &mut self.area_attacks,
                &mut self.damage_indicators,
            );
            processed += 1;
            if processed > network::MAX_MESSAGES_PER_FRAME {
                break; // Prevent infinite loop
            }
        }
    }

    /// Draw all game elements
    pub fn draw(&self) {
        RenderSystem::clear_screen();
        RenderSystem::draw_ui(&self.local_player, &self.remote_players);
        RenderSystem::draw_entities(
            &self.local_player,
            &self.remote_players,
            &self.boss,
            &self.bullets,
            &self.area_attacks,
            &self.damage_indicators,
        );
    }

    /// Check if the player wants to quit
    pub fn should_quit() -> bool {
        InputSystem::should_quit()
    }

    /// Send leave message when quitting
    pub fn send_leave_message(&self) {
        if let Some(sender) = &self.network_sender {
            let _ = sender.send(Payload::Leave(self.local_player.id));
        }
    }
}

/// Main game loop for the client
pub async fn run_client_game(
    network_sender: Sender<Payload>,
    network_receiver: Receiver<Payload>,
    player_id: u32,
) -> Result<()> {
    let mut game_state = GameState::new(player_id);
    game_state.set_network_sender(network_sender.clone());

    // Send join message
    let _ = network_sender.send(Payload::Join(game_state.local_player.id));

    loop {
        // Handle input and movement
        let moved = game_state.update_input();
        if moved {
            let move_payload = Payload::Move(
                game_state.local_player.id,
                game_state.local_player.x,
                game_state.local_player.y,
            );
            let _ = network_sender.send(move_payload);
        }

        // Update game entities
        game_state.update_entities();

        // Process network messages
        game_state.process_network_messages(&network_receiver);

        // Draw everything
        game_state.draw();

        // Check for quit
        if GameState::should_quit() {
            game_state.send_leave_message();
            break;
        }

        next_frame().await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let game_state = GameState::new(1);
        assert_eq!(game_state.local_player.id, 1);
        assert_eq!(game_state.remote_players.len(), 0);
        assert_eq!(game_state.bullets.len(), 0);
        assert!(game_state.boss.alive);
    }

    #[test]
    fn test_find_nearest_player_to_boss() {
        let game_state = GameState::new(1);
        let players = vec![Player::new(2, 100.0, 100.0), Player::new(3, 200.0, 200.0)];

        let nearest = game_state.find_nearest_player_to_boss(&players);
        assert!(nearest.is_some());
        // Boss starts at center, so player at 100,100 should be closer than 200,200
        assert_eq!(nearest.unwrap().id, 2);
    }
}
