use anyhow::Result;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Move(f32, f32),
    Join(u32),
    Leave(u32),
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
    pub last_sent_x: f32,
    pub last_sent_y: f32,
}

impl GameState {
    pub fn new(player_id: u32) -> Self {
        let local_player = Player {
            id: player_id,
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
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
        dx.abs() > 0.5 || dy.abs() > 0.5
    }

    pub fn mark_position_sent(&mut self) {
        self.last_sent_x = self.local_player.x;
        self.last_sent_y = self.local_player.y;
    }

    pub fn handle_network_message(&mut self, payload: &Payload) {
        match payload {
            Payload::Move(x, y) => {
                // Simple approach: one remote player
                if let Some(player) = self.remote_players.first_mut() {
                    player.x = *x;
                    player.y = *y;
                } else {
                    self.remote_players.push(Player {
                        id: 1,
                        x: *x,
                        y: *y,
                    });
                }
            }
            Payload::Join(id) => {
                println!("Player joined: {}", id);
            }
            Payload::Leave(id) => {
                println!("Player left: {}", id);
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

pub async fn run_client_game(
    network_sender: Sender<Payload>,
    network_receiver: Receiver<Payload>,
    player_id: u32,
) -> Result<()> {
    let mut game_state = GameState::new(player_id);
    let _ = network_sender.send(Payload::Join(game_state.local_player.id));

    loop {
        // Handle input and movement
        clear_background(WHITE);
        let moved = game_state.update_input();
        if moved && game_state.should_send_position() {
            let move_payload = Payload::Move(game_state.local_player.x, game_state.local_player.y);

            if network_sender.send(move_payload).is_ok() {
                game_state.mark_position_sent();
            }
        }

        // Process network messages
        while let Ok(payload) = network_receiver.try_recv() {
            game_state.handle_network_message(&payload);
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
