use anyhow::Result;
use dungeon::Payload;
use macroquad::prelude::*;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
pub struct GameState {
    pub local_player: Player,
    pub remote_players: Vec<Player>,
}

impl GameState {
    pub fn new(player_id: u32) -> Self {
        let local_player = Player {
            id: player_id,
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        };

        GameState {
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

    pub fn handle_network_message(&mut self, payload: &Payload) {
        match payload {
            Payload::Move(player_id, x, y) => {
                // Update position of the specific player
                if let Some(player) = self.remote_players.iter_mut().find(|p| p.id == *player_id) {
                    player.x = *x;
                    player.y = *y;
                } else {
                    // Player not found, add them (this can happen if we missed their Join message)
                    println!("Adding player {} from move message (missed join?)", player_id);
                    self.remote_players.push(Player {
                        id: *player_id,
                        x: *x,
                        y: *y,
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
                    });
                    println!("Player {} joined (total remote players: {})", id, self.remote_players.len());
                } else if *id == self.local_player.id {
                    println!("Ignoring join message for local player {}", id);
                } else {
                    println!("Player {} already exists, ignoring duplicate join", id);
                }
            }
            Payload::Leave(id) => {
                let initial_count = self.remote_players.len();
                self.remote_players.retain(|p| p.id != *id);
                let final_count = self.remote_players.len();
                if initial_count != final_count {
                    println!("Player {} left (remaining remote players: {})", id, final_count);
                } else {
                    println!("Received leave message for unknown player {}", id);
                }
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
        if moved {
            let move_payload = Payload::Move(game_state.local_player.id, game_state.local_player.x, game_state.local_player.y);
            let _ = network_sender.send(move_payload);
        }

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
