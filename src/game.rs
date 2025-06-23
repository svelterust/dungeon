use anyhow::Result;
use dungeon::Payload;
use macroquad::prelude::*;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub direction_x: f32,
    pub direction_y: f32,
    pub health: u32,
    pub max_health: u32,
    pub respawn_timer: f32,
    pub is_alive: bool,
    pub kills: u32,
}

#[derive(Debug, Clone)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub lifetime: f32,
    pub owner_id: u32,
    pub is_boss_bullet: bool,
}

#[derive(Debug, Clone)]
pub struct Boss {
    pub x: f32,
    pub y: f32,
    pub health: u32,
    pub max_health: u32,
    pub alive: bool,
    pub respawn_timer: f32,
    pub shoot_timer: f32,
    pub move_timer: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub power_timer: f32,
    pub shield_timer: f32,
    pub shield_active: bool,
    pub dash_timer: f32,
    pub is_dashing: bool,
    pub dash_target_x: f32,
    pub dash_target_y: f32,
}

impl Boss {
    pub fn new() -> Self {
        let x = screen_width() / 2.0;
        let y = 100.0;
        Boss {
            x,
            y,
            health: 100,
            max_health: 100,
            alive: true,
            respawn_timer: 0.0,
            shoot_timer: 0.0,
            move_timer: 0.0,
            target_x: x,
            target_y: y,
            power_timer: 0.0,
            shield_timer: 0.0,
            shield_active: false,
            dash_timer: 0.0,
            is_dashing: false,
            dash_target_x: x,
            dash_target_y: y,
        }
    }

    pub fn respawn(&mut self) {
        self.x = screen_width() / 2.0;
        self.y = 100.0;
        self.health = self.max_health;
        self.alive = true;
        self.respawn_timer = 0.0;
        self.shoot_timer = 0.0;
        self.move_timer = 0.0;
        self.target_x = self.x;
        self.target_y = self.y;
        self.power_timer = 0.0;
        self.shield_timer = 0.0;
        self.shield_active = false;
        self.dash_timer = 0.0;
        self.is_dashing = false;
        self.dash_target_x = self.x;
        self.dash_target_y = self.y;
    }

    pub fn update(&mut self, dt: f32, players: &[Player]) {
        if !self.alive {
            self.respawn_timer += dt;
            return;
        }

        // Update timers
        self.shoot_timer += dt;
        self.move_timer += dt;
        self.power_timer += dt;
        self.dash_timer += dt;
        if self.shield_active {
            self.shield_timer += dt;
            if self.shield_timer >= 3.0 {
                self.shield_active = false;
                self.shield_timer = 0.0;
            }
        }

        // Handle dashing
        if self.is_dashing {
            let dx = self.dash_target_x - self.x;
            let dy = self.dash_target_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 10.0 {
                let speed = 400.0 * dt; // Fast dash speed
                self.x += (dx / distance) * speed;
                self.y += (dy / distance) * speed;
            } else {
                self.is_dashing = false;
                self.dash_timer = 0.0;
            }
            return; // Skip normal movement while dashing
        }

        // Simple AI movement - move toward nearest player
        if self.move_timer >= 2.0 && !self.is_dashing {
            if let Some(nearest_player) = players.iter().filter(|p| p.is_alive).min_by_key(|p| {
                let dx = p.x - self.x;
                let dy = p.y - self.y;
                (dx * dx + dy * dy) as i32
            }) {
                self.target_x = nearest_player.x + (macroquad::rand::gen_range(-100.0, 100.0));
                self.target_y = nearest_player.y + (macroquad::rand::gen_range(-100.0, 100.0));

                // Keep boss on screen
                self.target_x = self.target_x.max(50.0).min(screen_width() - 50.0);
                self.target_y = self.target_y.max(50.0).min(screen_height() - 50.0);
            }
            self.move_timer = 0.0;
        }

        // Move toward target
        if !self.is_dashing {
            let dx = self.target_x - self.x;
            let dy = self.target_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 5.0 {
                let speed = 50.0 * dt;
                self.x += (dx / distance) * speed;
                self.y += (dy / distance) * speed;
            }
        }
    }

    pub fn should_shoot(&self) -> bool {
        self.alive && self.shoot_timer >= 1.5
    }

    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = 0.0;
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        if !self.alive || self.shield_active {
            return false;
        }

        if self.health <= damage {
            self.health = 0;
            self.alive = false;
            self.respawn_timer = 0.0;
            true // Boss died
        } else {
            self.health -= damage;
            false
        }
    }

    pub fn should_use_power(&self) -> bool {
        self.alive && self.power_timer >= 8.0 && !self.is_dashing
    }

    pub fn should_dash(&self) -> bool {
        self.alive && self.dash_timer >= 12.0 && !self.is_dashing
    }

    pub fn activate_shield(&mut self) {
        self.shield_active = true;
        self.shield_timer = 0.0;
    }

    pub fn start_dash(&mut self, target_x: f32, target_y: f32) {
        self.is_dashing = true;
        self.dash_target_x = target_x;
        self.dash_target_y = target_y;
        self.dash_timer = 0.0;
    }

    pub fn reset_power_timer(&mut self) {
        self.power_timer = 0.0;
    }

    pub fn should_respawn(&self) -> bool {
        !self.alive && self.respawn_timer >= 5.0
    }
}

impl Bullet {
    pub fn new(x: f32, y: f32, direction_x: f32, direction_y: f32, owner_id: u32) -> Self {
        let speed = 400.0;
        Self {
            x,
            y,
            velocity_x: direction_x * speed,
            velocity_y: direction_y * speed,
            lifetime: 3.0, // bullets last 3 seconds
            owner_id,
            is_boss_bullet: false,
        }
    }

    pub fn new_boss_bullet(x: f32, y: f32, direction_x: f32, direction_y: f32) -> Self {
        let speed = 300.0;
        Self {
            x,
            y,
            velocity_x: direction_x * speed,
            velocity_y: direction_y * speed,
            lifetime: 4.0, // boss bullets last longer
            owner_id: 0,   // Special ID for boss
            is_boss_bullet: true,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;
        self.lifetime -= dt;

        // Return true if bullet should be removed
        self.lifetime <= 0.0
            || self.x < 0.0
            || self.x > screen_width()
            || self.y < 0.0
            || self.y > screen_height()
    }
}

#[derive(Debug, Clone)]
pub struct AreaAttack {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
    pub max_time: f32,
}

#[derive(Debug, Clone)]
pub struct DamageIndicator {
    pub x: f32,
    pub y: f32,
    pub damage: u32,
    pub timer: f32,
    pub max_time: f32,
    pub from_player: bool,
}

impl AreaAttack {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            timer: 0.0,
            max_time: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.timer += dt;
        self.timer >= self.max_time
    }
}

impl DamageIndicator {
    pub fn new(x: f32, y: f32, damage: u32, from_player: bool) -> Self {
        Self {
            x,
            y,
            damage,
            timer: 0.0,
            max_time: 1.5,
            from_player,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.timer += dt;
        self.y -= 30.0 * dt; // Float upward
        self.timer >= self.max_time
    }
}

#[allow(dead_code)]
pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub boss: Boss,
    pub area_attacks: Vec<AreaAttack>,
    pub damage_indicators: Vec<DamageIndicator>,
    pub network_sender: Option<Sender<Payload>>,
}

impl GameState {
    pub fn new(player_id: u32) -> Self {
        let local_player = Player {
            id: player_id,
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            direction_x: 0.0,
            direction_y: -1.0, // Initially facing up
            health: 100,
            max_health: 100,
            respawn_timer: 0.0,
            is_alive: true,
            kills: 0,
        };

        GameState {
            local_player,
            remote_players: Vec::new(),
            bullets: Vec::new(),
            boss: Boss::new(),
            area_attacks: Vec::new(),
            damage_indicators: Vec::new(),
            network_sender: None,
        }
    }

    pub fn set_network_sender(&mut self, sender: Sender<Payload>) {
        self.network_sender = Some(sender);
    }

    pub fn update_input(&mut self) -> bool {
        let mut moved = false;

        // Handle respawning
        if !self.local_player.is_alive {
            let dt = get_frame_time();
            self.local_player.respawn_timer += dt;
            if self.local_player.respawn_timer >= 5.0 {
                // Respawn player directly without borrowing issues
                self.local_player.x = macroquad::rand::gen_range(50.0, screen_width() - 50.0);
                self.local_player.y = macroquad::rand::gen_range(screen_height() / 2.0, screen_height() - 50.0);
                self.local_player.health = self.local_player.max_health;
                self.local_player.is_alive = true;
                self.local_player.respawn_timer = 0.0;
                
                if let Some(sender) = &self.network_sender {
                    let _ = sender.send(Payload::PlayerRespawn(
                        self.local_player.id,
                        self.local_player.x,
                        self.local_player.y,
                    ));
                }
            }
            return false;
        }

        // Only allow movement if player is alive
        if !self.local_player.is_alive {
            return false;
        }

        let speed = 200.0 * get_frame_time();
        let mut new_direction_x = self.local_player.direction_x;
        let mut new_direction_y = self.local_player.direction_y;
        let mut direction_changed = false;

        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.local_player.x -= speed;
            new_direction_x = -1.0;
            new_direction_y = 0.0;
            moved = true;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.local_player.x += speed;
            new_direction_x = 1.0;
            new_direction_y = 0.0;
            moved = true;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            self.local_player.y -= speed;
            new_direction_x = 0.0;
            new_direction_y = -1.0;
            moved = true;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.local_player.y += speed;
            new_direction_x = 0.0;
            new_direction_y = 1.0;
            moved = true;
        }

        // Check if direction changed
        if new_direction_x != self.local_player.direction_x || new_direction_y != self.local_player.direction_y {
            direction_changed = true;
            self.local_player.direction_x = new_direction_x;
            self.local_player.direction_y = new_direction_y;
        }

        // Send direction update to network if direction changed
        if direction_changed {
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::PlayerDirection(
                    self.local_player.id,
                    self.local_player.direction_x,
                    self.local_player.direction_y,
                ));
            }
        }

        // Clamp to screen bounds
        self.local_player.x = self.local_player.x.max(15.0).min(screen_width() - 15.0);
        self.local_player.y = self.local_player.y.max(15.0).min(screen_height() - 15.0);

        // Handle shooting (only if alive)
        if is_key_pressed(KeyCode::Space) && self.local_player.is_alive {
            self.shoot_bullet();
        }

        moved
    }

    pub fn shoot_bullet(&mut self) {
        let bullet = Bullet::new(
            self.local_player.x,
            self.local_player.y,
            self.local_player.direction_x,
            self.local_player.direction_y,
            self.local_player.id,
        );
        self.bullets.push(bullet);

        // Send bullet to network
        if let Some(sender) = &self.network_sender {
            let _ = sender.send(Payload::Shoot(
                self.local_player.id,
                self.local_player.x,
                self.local_player.y,
                self.local_player.direction_x,
                self.local_player.direction_y,
            ));
        }
    }

    pub fn update_bullets(&mut self) {
        let dt = get_frame_time();
        self.bullets.retain_mut(|bullet| !bullet.update(dt));

        // Update area attacks
        self.area_attacks.retain_mut(|attack| !attack.update(dt));

        // Update damage indicators
        self.damage_indicators.retain_mut(|indicator| !indicator.update(dt));

        // Update remote player respawn timers
        for player in &mut self.remote_players {
            if !player.is_alive {
                player.respawn_timer += dt;
                if player.respawn_timer >= 5.0 {
                    // Remote players handle their own respawning
                    // We just update the timer here
                }
            }
        }

        // Check bullet collisions
        self.check_bullet_collisions();
    }

    pub fn check_bullet_collisions(&mut self) {
        let mut bullets_to_remove = Vec::new();

        for (i, bullet) in self.bullets.iter().enumerate() {
            // Check boss bullets hitting players
            if bullet.is_boss_bullet {
                // Check local player
                if self.local_player.is_alive && self.local_player.health > 0 {
                    let dx = bullet.x - self.local_player.x;
                    let dy = bullet.y - self.local_player.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= 18.0 {
                        // Player radius (15) + bullet radius (3)
                        self.local_player.health = self.local_player.health.saturating_sub(10);
                        bullets_to_remove.push(i);

                        if self.local_player.health == 0 {
                            self.local_player.is_alive = false;
                            self.local_player.respawn_timer = 0.0;
                        }

                        // Add damage indicator
                        self.damage_indicators.push(DamageIndicator::new(
                            self.local_player.x,
                            self.local_player.y,
                            10,
                            false,
                        ));

                        // Send health update to network
                        if let Some(sender) = &self.network_sender {
                            let _ = sender.send(Payload::PlayerHit(
                                self.local_player.id,
                                self.local_player.health,
                            ));
                        }
                    }
                }

                // Check remote players (they handle their own collisions)
            } else {
                // Check local player bullets hitting remote players (PvP)
                if bullet.owner_id == self.local_player.id {
                    for remote_player in &self.remote_players {
                        if remote_player.is_alive {
                            let dx = bullet.x - remote_player.x;
                            let dy = bullet.y - remote_player.y;
                            let distance = (dx * dx + dy * dy).sqrt();

                            if distance <= 18.0 {
                                // Player radius (15) + bullet radius (3)
                                bullets_to_remove.push(i);
                                
                                // Send damage to remote player
                                if let Some(sender) = &self.network_sender {
                                    let new_health = remote_player.health.saturating_sub(15);
                                    let _ = sender.send(Payload::PlayerHit(
                                        remote_player.id,
                                        new_health,
                                    ));
                                    
                                    // If this would kill the player, send kill notification
                                    if new_health == 0 {
                                        self.local_player.kills += 1;
                                        let _ = sender.send(Payload::PlayerKill(
                                            self.local_player.id,
                                            remote_player.id,
                                        ));
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                // Check player bullets hitting boss
                if self.boss.alive {
                    let dx = bullet.x - self.boss.x;
                    let dy = bullet.y - self.boss.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= 53.0 {
                        // Boss radius (50) + bullet radius (3)
                        let boss_died = self.boss.take_damage(10);
                        bullets_to_remove.push(i);

                        // Send boss hit to network
                        if let Some(sender) = &self.network_sender {
                            if boss_died {
                                let _ = sender.send(Payload::BossDead);
                            } else {
                                let _ = sender.send(Payload::BossHit(self.boss.health));
                            }
                        }
                    }
                }

                // Check player bullets hitting other players (PvP)
                // Only check if bullet is not from local player hitting local player
                if bullet.owner_id != self.local_player.id && self.local_player.is_alive {
                    let dx = bullet.x - self.local_player.x;
                    let dy = bullet.y - self.local_player.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= 18.0 {
                        // Player radius (15) + bullet radius (3)
                        let was_alive = self.local_player.is_alive;
                        self.local_player.health = self.local_player.health.saturating_sub(15);
                        bullets_to_remove.push(i);

                        if self.local_player.health == 0 {
                            self.local_player.is_alive = false;
                            self.local_player.respawn_timer = 0.0;
                            
                            // Award kill to shooter if player was alive
                            if was_alive {
                                if let Some(shooter) = self.remote_players.iter_mut().find(|p| p.id == bullet.owner_id) {
                                    shooter.kills += 1;
                                }
                            }
                        }

                        // Add PvP damage indicator
                        self.damage_indicators.push(DamageIndicator::new(
                            self.local_player.x,
                            self.local_player.y,
                            15,
                            true,
                        ));

                        // Send health update to network
                        if let Some(sender) = &self.network_sender {
                            let _ = sender.send(Payload::PlayerHit(
                                self.local_player.id,
                                self.local_player.health,
                            ));
                        }
                    }
                }
            }
        }

        // Remove bullets that hit something (in reverse order to maintain indices)
        for &i in bullets_to_remove.iter().rev() {
            self.bullets.remove(i);
        }
    }

    pub fn update_boss(&mut self) {
        let dt = get_frame_time();

        // Collect all alive players for boss AI
        let mut all_players = Vec::new();
        if self.local_player.is_alive {
            all_players.push(self.local_player.clone());
        }
        all_players.extend(self.remote_players.iter().filter(|p| p.is_alive).cloned());

        self.boss.update(dt, &all_players);

        // Handle boss powers
        if self.boss.should_use_power() && !all_players.is_empty() {
            let power_type = macroquad::rand::gen_range(0, 3);
            match power_type {
                0 => self.boss_multi_shot(&all_players),
                1 => self.boss_area_attack(&all_players),
                2 => self.boss_activate_shield(),
                _ => {}
            }
            self.boss.reset_power_timer();
        }

        // Handle boss dash
        if self.boss.should_dash() && !all_players.is_empty() {
            if let Some(target_player) = all_players.iter().min_by_key(|p| {
                let dx = p.x - self.boss.x;
                let dy = p.y - self.boss.y;
                (dx * dx + dy * dy) as i32
            }) {
                self.boss.start_dash(target_player.x, target_player.y);
                if let Some(sender) = &self.network_sender {
                    let _ = sender.send(Payload::BossDash(target_player.x, target_player.y));
                }
            }
        }

        // Handle boss shooting
        if self.boss.should_shoot() && !all_players.is_empty() && !self.boss.is_dashing {
            // Find nearest player to shoot at
            if let Some(target_player) = all_players.iter().min_by_key(|p| {
                let dx = p.x - self.boss.x;
                let dy = p.y - self.boss.y;
                (dx * dx + dy * dy) as i32
            }) {
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

        // Handle boss respawn
        if self.boss.should_respawn() {
            self.boss.respawn();
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::BossSpawn(self.boss.x, self.boss.y));
            }
        }
    }

    pub fn boss_multi_shot(&mut self, players: &[Player]) {
        if let Some(target_player) = players.iter().min_by_key(|p| {
            let dx = p.x - self.boss.x;
            let dy = p.y - self.boss.y;
            (dx * dx + dy * dy) as i32
        }) {
            let mut directions = Vec::new();

            // Create 5 bullets in a spread pattern
            for i in -2..=2 {
                let angle_offset = i as f32 * 0.2;
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

    pub fn boss_area_attack(&mut self, players: &[Player]) {
        if let Some(target_player) = players.iter().min_by_key(|p| {
            let dx = p.x - self.boss.x;
            let dy = p.y - self.boss.y;
            (dx * dx + dy * dy) as i32
        }) {
            // Create area damage
            let area_center_x = target_player.x;
            let area_center_y = target_player.y;

            // Check if local player is in area
            if self.local_player.is_alive {
                let dx = self.local_player.x - area_center_x;
                let dy = self.local_player.y - area_center_y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= 100.0 {
                    // Area damage radius
                    self.local_player.health = self.local_player.health.saturating_sub(20);
                    if self.local_player.health == 0 {
                        self.local_player.is_alive = false;
                        self.local_player.respawn_timer = 0.0;
                    }

                    if let Some(sender) = &self.network_sender {
                        let _ = sender.send(Payload::PlayerHit(
                            self.local_player.id,
                            self.local_player.health,
                        ));
                    }
                }
            }

            // Add visual area attack effect
            self.area_attacks
                .push(AreaAttack::new(area_center_x, area_center_y));

            // Send area attack to network
            if let Some(sender) = &self.network_sender {
                let _ = sender.send(Payload::BossAreaAttack(area_center_x, area_center_y));
            }
        }
    }

    pub fn boss_activate_shield(&mut self) {
        self.boss.activate_shield();
        if let Some(sender) = &self.network_sender {
            let _ = sender.send(Payload::BossShield(true));
        }
    }

    pub fn handle_network_message(&mut self, payload: &Payload) {
        match payload {
            Payload::Move(player_id, x, y) => {
                // Update position of the specific player
                if let Some(player) = self.remote_players.iter_mut().find(|p| p.id == *player_id) {
                    player.x = *x;
                    player.y = *y;
                } else {
                    // Player not found, add them (this can happen if we missed their Join message)
                    println!("Adding player {player_id} from move message (missed join?)");
                    self.remote_players.push(Player {
                        id: *player_id,
                        x: *x,
                        y: *y,
                        direction_x: 0.0,
                        direction_y: -1.0,
                        health: 100,
                        max_health: 100,
                        respawn_timer: 0.0,
                        is_alive: true,
                        kills: 0,
                    });
                }
            }
            Payload::Join(id) => {
                // Add remote player if not already present
                if !self.remote_players.iter().any(|p| p.id == *id) && *id != self.local_player.id {
                    self.remote_players.push(Player {
                        id: *id,
                        x: screen_width() / 2.0,
                        y: screen_height() / 2.0,
                        direction_x: 0.0,
                        direction_y: -1.0,
                        health: 100,
                        max_health: 100,
                        respawn_timer: 0.0,
                        is_alive: true,
                        kills: 0,
                    });
                    println!(
                        "Player {} joined (total remote players: {})",
                        id,
                        self.remote_players.len()
                    );
                } else if *id == self.local_player.id {
                    println!("Ignoring join message for local player {id}");
                } else {
                    println!("Player {id} already exists, ignoring duplicate join");
                }
            }
            Payload::Leave(id) => {
                let initial_count = self.remote_players.len();
                self.remote_players.retain(|p| p.id != *id);
                let final_count = self.remote_players.len();
                if initial_count != final_count {
                    println!("Player {id} left (remaining remote players: {final_count})");
                } else {
                    println!("Received leave message for unknown player {id}");
                }
            }
            Payload::Shoot(player_id, x, y, direction_x, direction_y) => {
                // Only add bullets from other players
                if *player_id != self.local_player.id {
                    let bullet = Bullet::new(*x, *y, *direction_x, *direction_y, *player_id);
                    self.bullets.push(bullet);
                }
            }
            Payload::BossShoot(x, y, direction_x, direction_y) => {
                let bullet = Bullet::new_boss_bullet(*x, *y, *direction_x, *direction_y);
                self.bullets.push(bullet);
            }
            Payload::PlayerHit(player_id, new_health) => {
                if *player_id == self.local_player.id {
                    self.local_player.health = *new_health;
                    if self.local_player.health == 0 {
                        self.local_player.is_alive = false;
                        self.local_player.respawn_timer = 0.0;
                    }
                } else if let Some(player) =
                    self.remote_players.iter_mut().find(|p| p.id == *player_id)
                {
                    player.health = *new_health;
                    if player.health == 0 {
                        player.is_alive = false;
                        player.respawn_timer = 0.0;
                    }
                }
            }
            Payload::BossHit(new_health) => {
                self.boss.health = *new_health;
            }
            Payload::BossSpawn(x, y) => {
                self.boss.x = *x;
                self.boss.y = *y;
                self.boss.respawn();
            }
            Payload::BossDead => {
                self.boss.alive = false;
                self.boss.health = 0;
                self.boss.respawn_timer = 0.0;
            }
            Payload::BossMultiShoot(x, y, directions) => {
                for (direction_x, direction_y) in directions {
                    let bullet = Bullet::new_boss_bullet(*x, *y, *direction_x, *direction_y);
                    self.bullets.push(bullet);
                }
            }
            Payload::BossDash(target_x, target_y) => {
                self.boss.start_dash(*target_x, *target_y);
            }
            Payload::BossAreaAttack(center_x, center_y) => {
                // Add visual area attack effect
                self.area_attacks
                    .push(AreaAttack::new(*center_x, *center_y));

                // Check if local player is affected by area attack
                if self.local_player.is_alive {
                    let dx = self.local_player.x - center_x;
                    let dy = self.local_player.y - center_y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= 100.0 {
                        self.local_player.health = self.local_player.health.saturating_sub(20);
                        if self.local_player.health == 0 {
                            self.local_player.is_alive = false;
                            self.local_player.respawn_timer = 0.0;
                        }

                        // Add area damage indicator
                        self.damage_indicators.push(DamageIndicator::new(
                            self.local_player.x,
                            self.local_player.y,
                            20,
                            false,
                        ));
                    }
                }
            }
            Payload::BossShield(active) => {
                if *active {
                    self.boss.activate_shield();
                } else {
                    self.boss.shield_active = false;
                    self.boss.shield_timer = 0.0;
                }
            }
            Payload::PlayerRespawn(player_id, x, y) => {
                if *player_id == self.local_player.id {
                    // This shouldn't happen since we handle our own respawn locally
                } else if let Some(player) =
                    self.remote_players.iter_mut().find(|p| p.id == *player_id)
                {
                    player.x = *x;
                    player.y = *y;
                    player.health = player.max_health;
                    player.is_alive = true;
                    player.respawn_timer = 0.0;
                }
            }
            Payload::PlayerDirection(player_id, direction_x, direction_y) => {
                if let Some(player) = self.remote_players.iter_mut().find(|p| p.id == *player_id) {
                    player.direction_x = *direction_x;
                    player.direction_y = *direction_y;
                }
            }
            Payload::PlayerKill(killer_id, victim_id) => {
                // Update kill count for killer
                if *killer_id == self.local_player.id {
                    // Local player got a kill (already handled locally)
                } else if let Some(killer) = self.remote_players.iter_mut().find(|p| p.id == *killer_id) {
                    killer.kills += 1;
                }
                
                // Handle victim death
                if *victim_id == self.local_player.id {
                    // Local player was killed (already handled via PlayerHit)
                } else if let Some(victim) = self.remote_players.iter_mut().find(|p| p.id == *victim_id) {
                    victim.is_alive = false;
                    victim.respawn_timer = 0.0;
                    victim.health = 0;
                }
            }
        }
    }

    pub fn draw(&self) {
        // Calculate total players (local + remote)
        let total_players = 1 + self.remote_players.len();

        // Draw player count and kill count in top left corner
        let player_text = format!("Players Connected: {total_players}");
        draw_text(&player_text, 10.0, 30.0, 20.0, BLACK);
        
        let kills_text = format!("Your Kills: {}", self.local_player.kills);
        draw_text(&kills_text, 10.0, 50.0, 18.0, DARKBLUE);
        
        // Draw leaderboard
        draw_text("LEADERBOARD", 10.0, 90.0, 16.0, DARKGRAY);
        
        // Collect all players with their kills
        let mut players_with_kills = vec![(self.local_player.id, self.local_player.kills, "You".to_string())];
        for player in &self.remote_players {
            players_with_kills.push((player.id, player.kills, format!("Player {}", player.id)));
        }
        
        // Sort by kills (descending)
        players_with_kills.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Draw top 5 players
        for (i, (_, kills, name)) in players_with_kills.iter().take(5).enumerate() {
            let y_pos = 110.0 + (i as f32 * 16.0);
            let rank_text = format!("{}. {} - {} kills", i + 1, name, kills);
            let color = if name == "You" { DARKBLUE } else { DARKGRAY };
            draw_text(&rank_text, 10.0, y_pos, 14.0, color);
        }

        // Draw boss
        if self.boss.alive {
            // Draw boss with shield effect
            if self.boss.shield_active {
                draw_circle(
                    self.boss.x,
                    self.boss.y,
                    60.0,
                    Color::new(0.0, 0.5, 1.0, 0.3),
                );
                draw_circle_lines(self.boss.x, self.boss.y, 60.0, 3.0, BLUE);
            }

            // Draw boss dash effect
            if self.boss.is_dashing {
                draw_circle(
                    self.boss.x,
                    self.boss.y,
                    55.0,
                    Color::new(1.0, 0.5, 0.0, 0.4),
                );
            }

            draw_circle(self.boss.x, self.boss.y, 50.0, MAROON);
            draw_circle(self.boss.x, self.boss.y, 45.0, RED);

            // Draw boss health bar
            self.draw_health_bar(
                self.boss.x - 50.0,
                self.boss.y - 70.0,
                100.0,
                8.0,
                self.boss.health,
                self.boss.max_health,
                RED,
            );

            // Draw boss power indicators
            let power_time_left = 8.0 - self.boss.power_timer;
            if power_time_left > 0.0 && power_time_left <= 2.0 {
                let warning_text = "BOSS POWER INCOMING!";
                let text_width = measure_text(warning_text, None, 20, 1.0).width;
                draw_text(
                    warning_text,
                    screen_width() / 2.0 - text_width / 2.0,
                    50.0,
                    20.0,
                    ORANGE,
                );
            }

            // Draw dash warning
            let dash_time_left = 12.0 - self.boss.dash_timer;
            if dash_time_left > 0.0 && dash_time_left <= 2.0 {
                let dash_text = "BOSS DASH INCOMING!";
                let text_width = measure_text(dash_text, None, 18, 1.0).width;
                draw_text(
                    dash_text,
                    screen_width() / 2.0 - text_width / 2.0,
                    70.0,
                    18.0,
                    YELLOW,
                );
            }
        } else {
            // Draw respawn timer
            let respawn_time_left = 5.0 - self.boss.respawn_timer;
            if respawn_time_left > 0.0 {
                let respawn_text = format!("Boss respawning in: {respawn_time_left:.1}s");
                let text_width = measure_text(&respawn_text, None, 24, 1.0).width;
                draw_text(
                    &respawn_text,
                    screen_width() / 2.0 - text_width / 2.0,
                    100.0,
                    24.0,
                    DARKGREEN,
                );
            }
        }

        // Draw local player (blue) with direction indicator
        if self.local_player.is_alive {
            draw_circle(self.local_player.x, self.local_player.y, 15.0, BLUE);

            // Draw direction arrow for local player
            let arrow_length = 25.0;
            let arrow_end_x = self.local_player.x + self.local_player.direction_x * arrow_length;
            let arrow_end_y = self.local_player.y + self.local_player.direction_y * arrow_length;
            draw_line(
                self.local_player.x,
                self.local_player.y,
                arrow_end_x,
                arrow_end_y,
                2.0,
                DARKBLUE,
            );
        } else {
            // Draw ghost player with respawn timer
            draw_circle(
                self.local_player.x,
                self.local_player.y,
                15.0,
                Color::new(0.5, 0.5, 0.5, 0.5),
            );

            let respawn_time_left = 5.0 - self.local_player.respawn_timer;
            if respawn_time_left > 0.0 {
                let respawn_text = format!("Respawning in: {:.1}s", respawn_time_left);
                let text_width = measure_text(&respawn_text, None, 16, 1.0).width;
                draw_text(
                    &respawn_text,
                    self.local_player.x - text_width / 2.0,
                    self.local_player.y - 25.0,
                    16.0,
                    WHITE,
                );
            }
        }

        // Draw local player health bar (only if alive)
        if self.local_player.is_alive {
            self.draw_health_bar(
                self.local_player.x - 20.0,
                self.local_player.y - 25.0,
                40.0,
                4.0,
                self.local_player.health,
                self.local_player.max_health,
                BLUE,
            );
        }

        // Draw remote players (red)
        for player in &self.remote_players {
            if player.is_alive {
                draw_circle(player.x, player.y, 15.0, RED);
                
                // Draw direction arrow for remote players
                let arrow_length = 25.0;
                let arrow_end_x = player.x + player.direction_x * arrow_length;
                let arrow_end_y = player.y + player.direction_y * arrow_length;
                draw_line(
                    player.x,
                    player.y,
                    arrow_end_x,
                    arrow_end_y,
                    2.0,
                    MAROON,
                );
                
                // Draw remote player health bar
                self.draw_health_bar(
                    player.x - 20.0,
                    player.y - 25.0,
                    40.0,
                    4.0,
                    player.health,
                    player.max_health,
                    RED,
                );
                
                // Draw kill count above health bar
                let kill_text = format!("K:{}", player.kills);
                let text_width = measure_text(&kill_text, None, 12, 1.0).width;
                draw_text(
                    &kill_text,
                    player.x - text_width / 2.0,
                    player.y - 35.0,
                    12.0,
                    WHITE,
                );
            } else {
                // Draw ghost remote player
                draw_circle(player.x, player.y, 15.0, Color::new(0.8, 0.2, 0.2, 0.5));

                let respawn_time_left = 5.0 - player.respawn_timer;
                if respawn_time_left > 0.0 {
                    let respawn_text = format!("Respawning: {:.1}s", respawn_time_left);
                    let text_width = measure_text(&respawn_text, None, 14, 1.0).width;
                    draw_text(
                        &respawn_text,
                        player.x - text_width / 2.0,
                        player.y - 25.0,
                        14.0,
                        WHITE,
                    );
                }
            }
        }

        // Draw area attacks
        for area_attack in &self.area_attacks {
            let progress = area_attack.timer / area_attack.max_time;
            let radius = 100.0 * (1.0 - progress);
            let alpha = 1.0 - progress;

            // Draw expanding circle
            draw_circle(
                area_attack.x,
                area_attack.y,
                radius,
                Color::new(1.0, 0.3, 0.0, alpha * 0.3),
            );
            draw_circle_lines(
                area_attack.x,
                area_attack.y,
                radius,
                3.0,
                Color::new(1.0, 0.5, 0.0, alpha),
            );

            // Draw warning at center
            if progress < 0.5 {
                let warning_alpha = (0.5 - progress) * 2.0;
                draw_circle(
                    area_attack.x,
                    area_attack.y,
                    10.0,
                    Color::new(1.0, 0.0, 0.0, warning_alpha),
                );
            }
        }

        // Draw bullets
        for bullet in &self.bullets {
            if bullet.is_boss_bullet {
                draw_circle(bullet.x, bullet.y, 5.0, MAROON);
                draw_circle(bullet.x, bullet.y, 3.0, ORANGE); // Inner glow
            } else if bullet.owner_id == self.local_player.id {
                draw_circle(bullet.x, bullet.y, 3.0, DARKBLUE);
            } else {
                draw_circle(bullet.x, bullet.y, 3.0, DARKPURPLE);
            }
        }

        // Draw damage indicators
        for indicator in &self.damage_indicators {
            let progress = indicator.timer / indicator.max_time;
            let alpha = 1.0 - progress;
            
            let color = if indicator.from_player {
                Color::new(1.0, 0.4, 0.0, alpha) // Orange for PvP damage
            } else {
                Color::new(1.0, 0.0, 0.0, alpha) // Red for boss damage
            };
            
            let damage_text = format!("-{}", indicator.damage);
            let text_width = measure_text(&damage_text, None, 18, 1.0).width;
            draw_text(
                &damage_text,
                indicator.x - text_width / 2.0,
                indicator.y,
                18.0,
                color,
            );
        }

        // Draw controls info in bottom left
        draw_text(
            "WASD/Arrow Keys: Move",
            10.0,
            screen_height() - 40.0,
            16.0,
            DARKGRAY,
        );
        draw_text("Space: Shoot", 10.0, screen_height() - 20.0, 16.0, DARKGRAY);
        draw_text("ESC: Quit", 10.0, screen_height() - 60.0, 16.0, DARKGRAY);
        draw_text("PvP: Players can shoot each other!", 10.0, screen_height() - 80.0, 14.0, ORANGE);
        draw_text("Kill other players to climb the leaderboard!", 10.0, screen_height() - 100.0, 14.0, GOLD);
    }

    fn draw_health_bar(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current_health: u32,
        max_health: u32,
        _color: Color,
    ) {
        // Background
        draw_rectangle(x, y, width, height, BLACK);

        // Health bar
        let health_percentage = current_health as f32 / max_health as f32;
        let health_width = width * health_percentage;

        let health_color = if health_percentage > 0.6 {
            GREEN
        } else if health_percentage > 0.3 {
            YELLOW
        } else {
            RED
        };

        draw_rectangle(x, y, health_width, height, health_color);

        // Border
        draw_rectangle_lines(x, y, width, height, 1.0, WHITE);

        // Health text
        let health_text = format!("{current_health}/{max_health}");
        let text_size = 12.0;
        let text_width = measure_text(&health_text, None, text_size as u16, 1.0).width;
        draw_text(
            &health_text,
            x + width / 2.0 - text_width / 2.0,
            y + height + 12.0,
            text_size,
            WHITE,
        );
    }
}

pub async fn run_client_game(
    network_sender: Sender<Payload>,
    network_receiver: Receiver<Payload>,
    player_id: u32,
) -> Result<()> {
    let mut game_state = GameState::new(player_id);
    game_state.set_network_sender(network_sender.clone());
    let _ = network_sender.send(Payload::Join(game_state.local_player.id));

    loop {
        // Handle input and movement
        clear_background(WHITE);
        let moved = game_state.update_input();
        if moved {
            let move_payload = Payload::Move(
                game_state.local_player.id,
                game_state.local_player.x,
                game_state.local_player.y,
            );
            let _ = network_sender.send(move_payload);
        }

        // Update bullets
        game_state.update_bullets();

        // Update boss
        game_state.update_boss();

        // Process ALL network messages immediately
        let mut processed = 0;
        while let Ok(payload) = network_receiver.try_recv() {
            game_state.handle_network_message(&payload);
            processed += 1;
            if processed > 100 {
                break;
            } // Prevent infinite loop
        }

        // Draw game objects
        game_state.draw();

        // Quit to leave game
        if is_key_pressed(KeyCode::Escape) {
            let _ = network_sender.send(Payload::Leave(game_state.local_player.id));
            break;
        }
        next_frame().await;
    }
    Ok(())
}
