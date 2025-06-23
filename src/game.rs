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
    }

    pub fn update(&mut self, dt: f32, players: &[Player]) {
        if !self.alive {
            self.respawn_timer += dt;
            return;
        }

        // Update timers
        self.shoot_timer += dt;
        self.move_timer += dt;

        // Simple AI movement - move toward nearest player
        if self.move_timer >= 2.0 {
            if let Some(nearest_player) = players.iter().min_by_key(|p| {
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
        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 5.0 {
            let speed = 50.0 * dt;
            self.x += (dx / distance) * speed;
            self.y += (dy / distance) * speed;
        }
    }

    pub fn should_shoot(&self) -> bool {
        self.alive && self.shoot_timer >= 1.5
    }

    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = 0.0;
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        if !self.alive {
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

#[allow(dead_code)]
pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub boss: Boss,
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
        };

        GameState {
            local_player,
            remote_players: Vec::new(),
            bullets: Vec::new(),
            boss: Boss::new(),
            network_sender: None,
        }
    }

    pub fn set_network_sender(&mut self, sender: Sender<Payload>) {
        self.network_sender = Some(sender);
    }

    pub fn update_input(&mut self) -> bool {
        let mut moved = false;

        // Only allow movement if player is alive
        if self.local_player.health == 0 {
            return false;
        }

        let speed = 200.0 * get_frame_time();
        let mut new_direction_x = self.local_player.direction_x;
        let mut new_direction_y = self.local_player.direction_y;

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

        // Update direction if moved
        if moved {
            self.local_player.direction_x = new_direction_x;
            self.local_player.direction_y = new_direction_y;
        }

        // Clamp to screen bounds
        self.local_player.x = self.local_player.x.max(15.0).min(screen_width() - 15.0);
        self.local_player.y = self.local_player.y.max(15.0).min(screen_height() - 15.0);

        // Handle shooting (only if alive)
        if is_key_pressed(KeyCode::Space) && self.local_player.health > 0 {
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

        // Check bullet collisions
        self.check_bullet_collisions();
    }

    pub fn check_bullet_collisions(&mut self) {
        let mut bullets_to_remove = Vec::new();

        for (i, bullet) in self.bullets.iter().enumerate() {
            // Check boss bullets hitting players
            if bullet.is_boss_bullet {
                // Check local player
                let dx = bullet.x - self.local_player.x;
                let dy = bullet.y - self.local_player.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= 18.0 {
                    // Player radius (15) + bullet radius (3)
                    if self.local_player.health > 0 {
                        self.local_player.health = self.local_player.health.saturating_sub(10);
                        bullets_to_remove.push(i);

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
            }
        }

        // Remove bullets that hit something (in reverse order to maintain indices)
        for &i in bullets_to_remove.iter().rev() {
            self.bullets.remove(i);
        }
    }

    pub fn update_boss(&mut self) {
        let dt = get_frame_time();

        // Collect all players for boss AI
        let mut all_players = vec![self.local_player.clone()];
        all_players.extend(self.remote_players.iter().cloned());

        self.boss.update(dt, &all_players);

        // Handle boss shooting
        if self.boss.should_shoot() && !all_players.is_empty() {
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
                } else if let Some(player) =
                    self.remote_players.iter_mut().find(|p| p.id == *player_id)
                {
                    player.health = *new_health;
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
        }
    }

    pub fn draw(&self) {
        // Calculate total players (local + remote)
        let total_players = 1 + self.remote_players.len();

        // Draw player count in top left corner
        let player_text = format!("Players Connected: {total_players}");
        draw_text(&player_text, 10.0, 30.0, 20.0, BLACK);

        // Draw boss
        if self.boss.alive {
            draw_circle(self.boss.x, self.boss.y, 50.0, DARKGREEN);
            draw_circle(self.boss.x, self.boss.y, 45.0, RED);

            // Draw boss health bar
            self.draw_health_bar(
                self.boss.x - 50.0,
                self.boss.y - 70.0,
                100.0,
                8.0,
                self.boss.health,
                self.boss.max_health,
            );
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
        let player_color = if self.local_player.health > 0 {
            BLUE
        } else {
            LIGHTGRAY
        };
        draw_circle(self.local_player.x, self.local_player.y, 15.0, player_color);

        // Draw direction arrow for local player
        if self.local_player.health > 0 {
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
        }

        // Draw local player health bar
        self.draw_health_bar(
            self.local_player.x - 20.0,
            self.local_player.y - 25.0,
            40.0,
            4.0,
            self.local_player.health,
            self.local_player.max_health,
        );

        // Draw remote players (red)
        for player in &self.remote_players {
            let player_color = if player.health > 0 { RED } else { LIGHTGRAY };
            draw_circle(player.x, player.y, 15.0, player_color);

            // Draw remote player health bar
            self.draw_health_bar(
                player.x - 20.0,
                player.y - 25.0,
                40.0,
                4.0,
                player.health,
                player.max_health,
            );
        }

        // Draw bullets
        for bullet in &self.bullets {
            if bullet.is_boss_bullet {
                draw_circle(bullet.x, bullet.y, 5.0, DARKGREEN);
            } else if bullet.owner_id == self.local_player.id {
                draw_circle(bullet.x, bullet.y, 3.0, DARKBLUE);
            } else {
                draw_circle(bullet.x, bullet.y, 3.0, DARKPURPLE);
            }
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
    }

    fn draw_health_bar(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current_health: u32,
        max_health: u32,
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
