use anyhow::Result;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    Move { x: f32, y: f32 },
    Join { name: String },
    Leave { name: String },
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub name: String,
}

pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub last_sent_x: f32,
    pub last_sent_y: f32,
}

#[allow(dead_code)]
impl GameState {
    pub fn new(player_name: &str) -> Self {
        let local_player = Player {
            id: "local".to_string(),
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            name: player_name.to_string(),
        };

        GameState {
            last_sent_x: local_player.x,
            last_sent_y: local_player.y,
            local_player,
            remote_players: Vec::new(),
        }
    }

    pub fn update_input(&mut self) -> bool {
        let mut moved = false;
        let speed = 200.0 * get_frame_time();

        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.local_player.x -= speed;
            moved = true;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.local_player.x += speed;
            moved = true;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            self.local_player.y -= speed;
            moved = true;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.local_player.y += speed;
            moved = true;
        }

        // Clamp to screen bounds
        self.local_player.x = self.local_player.x.max(15.0).min(screen_width() - 15.0);
        self.local_player.y = self.local_player.y.max(15.0).min(screen_height() - 15.0);

        moved
    }

    pub fn should_send_position(&self) -> bool {
        let dx = self.local_player.x - self.last_sent_x;
        let dy = self.local_player.y - self.last_sent_y;
        dx.abs() > 5.0 || dy.abs() > 5.0
    }

    pub fn mark_position_sent(&mut self) {
        self.last_sent_x = self.local_player.x;
        self.last_sent_y = self.local_player.y;
    }

    pub fn handle_network_message(&mut self, payload: &Payload) {
        match payload {
            Payload::Move { x, y } => {
                // For now, simple approach: one remote player
                // In a real game, you'd need player IDs
                if let Some(player) = self.remote_players.first_mut() {
                    player.x = *x;
                    player.y = *y;
                } else {
                    self.remote_players.push(Player {
                        id: "remote_1".to_string(),
                        x: *x,
                        y: *y,
                        name: "Remote Player".to_string(),
                    });
                }
            }
            Payload::Join { name } => {
                println!("Player joined: {}", name);
                // Could add to remote_players list here
            }
            Payload::Leave { name } => {
                println!("Player left: {}", name);
                // Could remove from remote_players list here
            }
        }
    }

    pub fn draw(&self) {
        // Draw local player (blue)
        draw_circle(self.local_player.x, self.local_player.y, 15.0, BLUE);

        // Draw remote players (red)
        for player in &self.remote_players {
            draw_circle(player.x, player.y, 15.0, RED);
        }
    }
}

#[allow(dead_code)]
pub async fn run_standalone_game() -> Result<()> {
    loop {
        clear_background(WHITE);

        // Original demo content
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 45.0, screen_height() - 45.0, 30.0, PURPLE);

        // Demo payload
        draw_text("STANDALONE MODE", 20.0, 30.0, 25.0, BLACK);
        draw_text("Press ESC to quit", 20.0, 60.0, 18.0, GRAY);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn run_client_game(
    network_sender: Sender<Payload>,
    network_receiver: Receiver<Payload>,
    server_address: &str,
) -> Result<()> {
    let mut game_state = GameState::new("Player");

    // Send initial join message
    let join_payload = Payload::Join {
        name: game_state.local_player.name.clone(),
    };
    if network_sender.send(join_payload).is_err() {
        eprintln!("Failed to send join message");
    }

    loop {
        clear_background(WHITE);

        // Handle input and movement
        let moved = game_state.update_input();

        // Send position if moved significantly
        if moved && game_state.should_send_position() {
            let move_payload = Payload::Move {
                x: game_state.local_player.x,
                y: game_state.local_player.y,
            };

            if network_sender.send(move_payload).is_ok() {
                game_state.mark_position_sent();
            }
        }

        // Process network messages
        while let Ok(payload) = network_receiver.try_recv() {
            game_state.handle_network_message(&payload);
        }

        // Draw UI
        draw_text("CLIENT MODE", 20.0, 30.0, 25.0, BLACK);
        draw_text(
            &format!("Connected to: {}", server_address),
            20.0,
            60.0,
            20.0,
            GRAY,
        );
        draw_text("Use WASD or arrows to move", 20.0, 90.0, 18.0, GRAY);
        draw_text("Press ESC to quit", 20.0, 110.0, 18.0, GRAY);

        // Draw game objects
        game_state.draw();

        if is_key_pressed(KeyCode::Escape) {
            // Send leave message
            let leave_payload = Payload::Leave {
                name: game_state.local_player.name.clone(),
            };
            let _ = network_sender.send(leave_payload);
            break;
        }
        next_frame().await;
    }
    Ok(())
}
