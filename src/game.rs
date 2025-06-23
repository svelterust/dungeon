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
}

#[derive(Debug, Clone)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub lifetime: f32,
    pub owner_id: u32,
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
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;
        self.lifetime -= dt;

        // Return true if bullet should be removed
        self.lifetime <= 0.0 ||
        self.x < 0.0 || self.x > screen_width() ||
        self.y < 0.0 || self.y > screen_height()
    }


}

#[allow(dead_code)]
pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub bullets: Vec<Bullet>,
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
        };

        GameState {
            local_player,
            remote_players: Vec::new(),
            bullets: Vec::new(),
            network_sender: None,
        }
    }

    pub fn set_network_sender(&mut self, sender: Sender<Payload>) {
        self.network_sender = Some(sender);
    }

    pub fn update_input(&mut self) -> bool {
        let mut moved = false;
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

        // Handle shooting
        if is_key_pressed(KeyCode::Space) {
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
                    });
                    println!("Player {} joined (total remote players: {})", id, self.remote_players.len());
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
        }
    }

    pub fn draw(&self) {
        // Calculate total players (local + remote)
        let total_players = 1 + self.remote_players.len();
        
        // Draw player count in top left corner
        let player_text = format!("Players Connected: {}", total_players);
        draw_text(&player_text, 10.0, 30.0, 20.0, BLACK);
        
        // Draw local player (blue) with direction indicator
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

        // Draw remote players (red)
        for player in &self.remote_players {
            draw_circle(player.x, player.y, 15.0, RED);
        }

        // Draw bullets (local bullets in yellow, remote bullets in orange)
        for bullet in &self.bullets {
            if bullet.owner_id == self.local_player.id {
                draw_circle(bullet.x, bullet.y, 3.0, YELLOW);
            } else {
                draw_circle(bullet.x, bullet.y, 3.0, ORANGE);
            }
        }

        // Draw controls info in bottom left
        draw_text("WASD/Arrow Keys: Move", 10.0, screen_height() - 40.0, 16.0, DARKGRAY);
        draw_text("Space: Shoot", 10.0, screen_height() - 20.0, 16.0, DARKGRAY);
        draw_text("ESC: Quit", 10.0, screen_height() - 60.0, 16.0, DARKGRAY);
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
            let move_payload = Payload::Move(game_state.local_player.id, game_state.local_player.x, game_state.local_player.y);
            let _ = network_sender.send(move_payload);
        }

        // Update bullets
        game_state.update_bullets();

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