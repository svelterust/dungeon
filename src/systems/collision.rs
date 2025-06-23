//! Collision detection system for handling all game collisions

use crate::Payload;
use crate::constants::{boss, player};
use crate::entities::{AreaAttack, Boss, Bullet, DamageIndicator, Player};
use std::sync::mpsc::Sender;

/// Handles all collision detection in the game
pub struct CollisionSystem {
    bullets_to_remove: Vec<usize>,
}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {
            bullets_to_remove: Vec::new(),
        }
    }

    /// Check all bullet collisions and handle damage
    pub fn check_bullet_collisions(
        &mut self,
        bullets: &mut Vec<Bullet>,
        local_player: &mut Player,
        remote_players: &mut Vec<Player>,
        boss: &mut Boss,
        damage_indicators: &mut Vec<DamageIndicator>,
        network_sender: &Option<Sender<Payload>>,
    ) {
        self.bullets_to_remove.clear();

        for (i, bullet) in bullets.iter().enumerate() {
            if self.handle_bullet_collision(
                bullet,
                local_player,
                remote_players,
                boss,
                damage_indicators,
                network_sender,
            ) {
                self.bullets_to_remove.push(i);
            }
        }

        // Remove bullets that hit something (in reverse order to maintain indices)
        for &i in self.bullets_to_remove.iter().rev() {
            bullets.remove(i);
        }
    }

    /// Handle collision for a single bullet
    fn handle_bullet_collision(
        &self,
        bullet: &Bullet,
        local_player: &mut Player,
        remote_players: &mut Vec<Player>,
        boss: &mut Boss,
        damage_indicators: &mut Vec<DamageIndicator>,
        network_sender: &Option<Sender<Payload>>,
    ) -> bool {
        if bullet.is_boss_bullet {
            self.handle_boss_bullet_collision(
                bullet,
                local_player,
                damage_indicators,
                network_sender,
            )
        } else {
            self.handle_player_bullet_collision(
                bullet,
                local_player,
                remote_players,
                boss,
                damage_indicators,
                network_sender,
            )
        }
    }

    /// Handle collisions for boss bullets
    fn handle_boss_bullet_collision(
        &self,
        bullet: &Bullet,
        local_player: &mut Player,
        damage_indicators: &mut Vec<DamageIndicator>,
        network_sender: &Option<Sender<Payload>>,
    ) -> bool {
        // Check collision with local player
        if local_player.is_alive
            && bullet.collides_with(local_player.x, local_player.y, player::RADIUS)
        {
            // Add damage indicator
            let damage = bullet.damage();
            damage_indicators.push(DamageIndicator::new(
                local_player.x,
                local_player.y,
                damage,
                false,
            ));

            // Send health update to network
            if let Some(sender) = network_sender {
                let _ = sender.send(Payload::PlayerHit(local_player.id, local_player.health));
            }
            return true;
        }

        false
    }

    /// Handle collisions for player bullets
    fn handle_player_bullet_collision(
        &self,
        bullet: &Bullet,
        local_player: &mut Player,
        remote_players: &mut Vec<Player>,
        boss: &mut Boss,
        damage_indicators: &mut Vec<DamageIndicator>,
        network_sender: &Option<Sender<Payload>>,
    ) -> bool {
        // Check local player bullet hitting remote players (PvP)
        if bullet.owner_id == local_player.id {
            if let Some(hit_player) = remote_players
                .iter()
                .find(|p| p.is_alive && bullet.collides_with(p.x, p.y, player::RADIUS))
            {
                // Send damage to remote player
                if let Some(sender) = network_sender {
                    let new_health = hit_player.health.saturating_sub(bullet.damage());
                    let _ = sender.send(Payload::PlayerHit(hit_player.id, new_health));

                    // If this would kill the player, send kill notification
                    if new_health == 0 {
                        local_player.kills += 1;
                        let _ = sender.send(Payload::PlayerKill(local_player.id, hit_player.id));
                    }
                }
                return true; // Remove bullet
            }
        }

        // Check remote player bullet hitting local player (PvP)
        if bullet.owner_id != local_player.id && local_player.is_alive {
            if bullet.collides_with(local_player.x, local_player.y, player::RADIUS) {
                let was_alive = local_player.is_alive;
                let damage = bullet.damage();
                let died = local_player.take_damage(damage);

                // Award kill to shooter if player was alive
                if was_alive && died {
                    if let Some(shooter) =
                        remote_players.iter_mut().find(|p| p.id == bullet.owner_id)
                    {
                        shooter.kills += 1;
                    }
                }

                // Add PvP damage indicator
                damage_indicators.push(DamageIndicator::new(
                    local_player.x,
                    local_player.y,
                    damage,
                    true,
                ));

                // Send health update to network
                if let Some(sender) = network_sender {
                    let _ = sender.send(Payload::PlayerHit(local_player.id, local_player.health));
                }

                return true; // Remove bullet
            }
        }

        // Check player bullets hitting boss
        if boss.alive && bullet.collides_with(boss.x, boss.y, boss::RADIUS) {
            let boss_died = boss.take_damage(bullet.damage());

            // Add damage indicator
            damage_indicators.push(DamageIndicator::new(boss.x, boss.y, bullet.damage(), false));

            // Send boss hit to network
            if let Some(sender) = network_sender {
                if boss_died {
                    let _ = sender.send(Payload::BossDead);
                } else {
                    let _ = sender.send(Payload::BossHit(boss.health));
                }
            }

            return true; // Remove bullet
        }

        false
    }

    /// Check area attack collisions
    pub fn check_area_attack_collisions(
        &self,
        area_attacks: &[AreaAttack],
        local_player: &mut Player,
        damage_indicators: &mut Vec<DamageIndicator>,
        network_sender: &Option<Sender<Payload>>,
    ) {
        for area_attack in area_attacks {
            if local_player.is_alive && area_attack.affects_point(local_player.x, local_player.y) {
                let damage = area_attack.damage();
                local_player.take_damage(damage);

                // Add damage indicator
                damage_indicators.push(DamageIndicator::new(
                    local_player.x,
                    local_player.y,
                    damage,
                    false,
                ));

                // Send health update to network
                if let Some(sender) = network_sender {
                    let _ = sender.send(Payload::PlayerHit(local_player.id, local_player.health));
                }
            }
        }
    }
}

impl Default for CollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_system_creation() {
        let system = CollisionSystem::new();
        assert_eq!(system.bullets_to_remove.len(), 0);
    }

    #[test]
    fn test_bullet_player_collision() {
        let player = Player::new(1, 100.0, 100.0);
        let bullet = Bullet::new(100.0, 100.0, 1.0, 0.0, 2);

        // Test collision detection
        assert!(bullet.collides_with(player.x, player.y, player::RADIUS));
    }
}
