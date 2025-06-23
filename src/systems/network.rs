//! Network message handling system for multiplayer communication

use crate::Payload;
use crate::entities::{AreaAttack, Boss, Bullet, DamageIndicator, Player};
use macroquad::prelude::*;

/// Handles processing of network messages and game state synchronization
pub struct NetworkSystem;

impl NetworkSystem {
    /// Process a single network message and update game state accordingly
    pub fn handle_message(
        payload: &Payload,
        local_player: &mut Player,
        remote_players: &mut Vec<Player>,
        boss: &mut Boss,
        bullets: &mut Vec<Bullet>,
        area_attacks: &mut Vec<AreaAttack>,
        damage_indicators: &mut Vec<DamageIndicator>,
    ) {
        match payload {
            Payload::Move(player_id, x, y) => {
                Self::handle_player_move(*player_id, *x, *y, local_player, remote_players);
            }
            Payload::Join(id) => {
                Self::handle_player_join(*id, local_player, remote_players);
            }
            Payload::Leave(id) => {
                Self::handle_player_leave(*id, remote_players);
            }
            Payload::Shoot(player_id, x, y, direction_x, direction_y) => {
                Self::handle_player_shoot(
                    *player_id,
                    *x,
                    *y,
                    *direction_x,
                    *direction_y,
                    local_player,
                    bullets,
                );
            }
            Payload::BossShoot(x, y, direction_x, direction_y) => {
                Self::handle_boss_shoot(*x, *y, *direction_x, *direction_y, bullets);
            }
            Payload::PlayerHit(player_id, new_health, damage) => {
                Self::handle_player_hit(*player_id, *new_health, *damage, local_player, remote_players, damage_indicators);
            }
            Payload::BossHit(new_health) => {
                Self::handle_boss_hit(*new_health, boss);
            }
            Payload::BossSpawn(x, y) => {
                Self::handle_boss_spawn(*x, *y, boss);
            }
            Payload::BossDead => {
                Self::handle_boss_death(boss);
            }
            Payload::BossMultiShoot(x, y, directions) => {
                Self::handle_boss_multi_shoot(*x, *y, directions, bullets);
            }
            Payload::BossDash(target_x, target_y) => {
                Self::handle_boss_dash(*target_x, *target_y, boss);
            }
            Payload::BossAreaAttack(center_x, center_y) => {
                Self::handle_boss_area_attack(
                    *center_x,
                    *center_y,
                    local_player,
                    area_attacks,
                    damage_indicators,
                );
            }
            Payload::BossShield(active) => {
                Self::handle_boss_shield(*active, boss);
            }
            Payload::PlayerRespawn(player_id, x, y) => {
                Self::handle_player_respawn(*player_id, *x, *y, local_player, remote_players);
            }
            Payload::PlayerDirection(player_id, direction_x, direction_y) => {
                Self::handle_player_direction(
                    *player_id,
                    *direction_x,
                    *direction_y,
                    remote_players,
                );
            }
            Payload::PlayerKill(killer_id, victim_id) => {
                Self::handle_player_kill(*killer_id, *victim_id, local_player, remote_players);
            }
        }
    }

    /// Handle player movement update
    fn handle_player_move(
        player_id: u32,
        x: f32,
        y: f32,
        local_player: &Player,
        remote_players: &mut Vec<Player>,
    ) {
        if let Some(player) = remote_players.iter_mut().find(|p| p.id == player_id) {
            player.x = x;
            player.y = y;
        } else if player_id != local_player.id {
            // Player not found, add them (missed join message)
            println!("Adding player {player_id} from move message (missed join?)");
            let new_player = Player::new(player_id, x, y);
            remote_players.push(new_player);
        }
    }

    /// Handle player joining the game
    fn handle_player_join(player_id: u32, local_player: &Player, remote_players: &mut Vec<Player>) {
        // Don't add local player or duplicates
        if player_id == local_player.id {
            println!("Ignoring join message for local player {player_id}");
            return;
        }

        if remote_players.iter().any(|p| p.id == player_id) {
            println!("Player {player_id} already exists, ignoring duplicate join");
            return;
        }

        let new_player = Player::new_at_center(player_id);
        remote_players.push(new_player);
        println!(
            "Player {} joined (total remote players: {})",
            player_id,
            remote_players.len()
        );
    }

    /// Handle player leaving the game
    fn handle_player_leave(player_id: u32, remote_players: &mut Vec<Player>) {
        let initial_count = remote_players.len();
        remote_players.retain(|p| p.id != player_id);
        let final_count = remote_players.len();

        if initial_count != final_count {
            println!("Player {player_id} left (remaining remote players: {final_count})");
        } else {
            println!("Received leave message for unknown player {player_id}");
        }
    }

    /// Handle player shooting
    fn handle_player_shoot(
        player_id: u32,
        x: f32,
        y: f32,
        direction_x: f32,
        direction_y: f32,
        local_player: &Player,
        bullets: &mut Vec<Bullet>,
    ) {
        // Only add bullets from other players
        if player_id != local_player.id {
            let bullet = Bullet::new(x, y, direction_x, direction_y, player_id);
            bullets.push(bullet);
        }
    }

    /// Handle boss shooting
    fn handle_boss_shoot(
        x: f32,
        y: f32,
        direction_x: f32,
        direction_y: f32,
        bullets: &mut Vec<Bullet>,
    ) {
        let bullet = Bullet::new_boss_bullet(x, y, direction_x, direction_y);
        bullets.push(bullet);
    }

    /// Handle player taking damage
    fn handle_player_hit(
        player_id: u32,
        new_health: u32,
        damage: u32,
        local_player: &mut Player,
        remote_players: &mut [Player],
        damage_indicators: &mut Vec<DamageIndicator>,
    ) {
        if player_id == local_player.id {
            local_player.health = new_health;
            if local_player.health == 0 {
                local_player.is_alive = false;
                local_player.respawn_timer = 0.0;
            }
        } else if let Some(player) = remote_players.iter_mut().find(|p| p.id == player_id) {
            player.health = new_health;
            if player.health == 0 {
                player.is_alive = false;
                player.respawn_timer = 0.0;
            }
            
            // Add damage indicator for remote player
            damage_indicators.push(DamageIndicator::new(
                player.x,
                player.y,
                damage,
                false,
            ));
        }
    }

    /// Handle boss taking damage
    fn handle_boss_hit(new_health: u32, boss: &mut Boss) {
        boss.health = new_health;
    }

    /// Handle boss spawning
    fn handle_boss_spawn(x: f32, y: f32, boss: &mut Boss) {
        boss.x = x;
        boss.y = y;
        boss.respawn();
    }

    /// Handle boss death
    fn handle_boss_death(boss: &mut Boss) {
        boss.alive = false;
        boss.health = 0;
        boss.respawn_timer = 0.0;
    }

    /// Handle boss multi-shot attack
    fn handle_boss_multi_shoot(
        x: f32,
        y: f32,
        directions: &[(f32, f32)],
        bullets: &mut Vec<Bullet>,
    ) {
        for (direction_x, direction_y) in directions {
            let bullet = Bullet::new_boss_bullet(x, y, *direction_x, *direction_y);
            bullets.push(bullet);
        }
    }

    /// Handle boss dash attack
    fn handle_boss_dash(target_x: f32, target_y: f32, boss: &mut Boss) {
        boss.start_dash(target_x, target_y);
    }

    /// Handle boss area attack
    fn handle_boss_area_attack(
        center_x: f32,
        center_y: f32,
        local_player: &mut Player,
        area_attacks: &mut Vec<AreaAttack>,
        damage_indicators: &mut Vec<DamageIndicator>,
    ) {
        // Add visual area attack effect
        area_attacks.push(AreaAttack::new(center_x, center_y));

        // Check if local player is affected
        if local_player.is_alive {
            let area_attack = AreaAttack::new(center_x, center_y);
            if area_attack.affects_point(local_player.x, local_player.y) {
                let damage = area_attack.damage();
                local_player.take_damage(damage);

                // Add damage indicator
                damage_indicators.push(DamageIndicator::new(
                    local_player.x,
                    local_player.y,
                    damage,
                    false,
                ));
            }
        }
    }

    /// Handle boss shield activation/deactivation
    fn handle_boss_shield(active: bool, boss: &mut Boss) {
        if active {
            boss.activate_shield();
        } else {
            boss.shield_active = false;
            boss.shield_timer = 0.0;
        }
    }

    /// Handle player respawn
    fn handle_player_respawn(
        player_id: u32,
        x: f32,
        y: f32,
        local_player: &Player,
        remote_players: &mut [Player],
    ) {
        // Local player handles their own respawn
        if player_id != local_player.id
            && let Some(player) = remote_players.iter_mut().find(|p| p.id == player_id)
        {
            player.x = x;
            player.y = y;
            player.health = player.max_health;
            player.is_alive = true;
            player.respawn_timer = 0.0;
        }
    }

    /// Handle player direction update
    fn handle_player_direction(
        player_id: u32,
        direction_x: f32,
        direction_y: f32,
        remote_players: &mut [Player],
    ) {
        if let Some(player) = remote_players.iter_mut().find(|p| p.id == player_id) {
            player.direction_x = direction_x;
            player.direction_y = direction_y;
        }
    }

    /// Handle player kill notification
    fn handle_player_kill(
        killer_id: u32,
        victim_id: u32,
        local_player: &mut Player,
        remote_players: &mut [Player],
    ) {
        // Update kill count for killer
        if killer_id == local_player.id {
            // Local player got a kill (already handled locally)
        } else if let Some(killer) = remote_players.iter_mut().find(|p| p.id == killer_id) {
            killer.kills += 1;
        }

        // Handle victim death
        if victim_id == local_player.id {
            // Local player was killed (already handled via PlayerHit)
        } else if let Some(victim) = remote_players.iter_mut().find(|p| p.id == victim_id) {
            victim.is_alive = false;
            victim.respawn_timer = 0.0;
            victim.health = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_system_creation() {
        // NetworkSystem is a unit struct, so just test that it exists
        let _system = NetworkSystem;
    }

    #[test]
    fn test_handle_player_join() {
        let local_player = Player::new(1, 100.0, 100.0);
        let mut remote_players = Vec::new();

        NetworkSystem::handle_player_join(2, &local_player, &mut remote_players);
        assert_eq!(remote_players.len(), 1);
        assert_eq!(remote_players[0].id, 2);

        // Test duplicate join
        NetworkSystem::handle_player_join(2, &local_player, &mut remote_players);
        assert_eq!(remote_players.len(), 1); // Should not add duplicate
    }

    #[test]
    fn test_handle_player_leave() {
        let mut remote_players = vec![Player::new(2, 100.0, 100.0), Player::new(3, 200.0, 200.0)];

        NetworkSystem::handle_player_leave(2, &mut remote_players);
        assert_eq!(remote_players.len(), 1);
        assert_eq!(remote_players[0].id, 3);
    }
}
