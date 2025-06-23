//! Player entity with core functionality

use crate::constants::{player, ui};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Player {
    /// Create a new player at the specified position
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            direction_x: 0.0,
            direction_y: -1.0, // Initially facing up
            health: player::MAX_HEALTH,
            max_health: player::MAX_HEALTH,
            respawn_timer: 0.0,
            is_alive: true,
            kills: 0,
        }
    }

    /// Create a player at screen center
    pub fn new_at_center(id: u32) -> Self {
        Self::new(id, screen_width() / 2.0, screen_height() / 2.0)
    }

    /// Update player's respawn timer
    pub fn update_respawn(&mut self, dt: f32) {
        if !self.is_alive {
            self.respawn_timer += dt;
        }
    }

    /// Check if player can respawn
    pub fn can_respawn(&self) -> bool {
        !self.is_alive && self.respawn_timer >= player::RESPAWN_TIME
    }

    /// Respawn the player at a random safe location
    pub fn respawn(&mut self) {
        self.x = macroquad::rand::gen_range(player::RADIUS, screen_width() - player::RADIUS);
        self.y = macroquad::rand::gen_range(screen_height() / 2.0, screen_height() - player::RADIUS);
        self.health = self.max_health;
        self.is_alive = true;
        self.respawn_timer = 0.0;
    }

    /// Move player by velocity, clamping to screen bounds
    pub fn move_by(&mut self, velocity_x: f32, velocity_y: f32) {
        self.x += velocity_x;
        self.y += velocity_y;
        self.clamp_to_screen();
    }

    /// Set player direction
    pub fn set_direction(&mut self, direction_x: f32, direction_y: f32) -> bool {
        let direction_changed = self.direction_x != direction_x || self.direction_y != direction_y;
        self.direction_x = direction_x;
        self.direction_y = direction_y;
        direction_changed
    }

    /// Clamp player position to screen bounds
    pub fn clamp_to_screen(&mut self) {
        let min_pos = player::RADIUS;
        self.x = self.x.clamp(min_pos, screen_width() - min_pos);
        self.y = self.y.clamp(min_pos, screen_height() - min_pos);
    }

    /// Take damage and return whether player died
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if !self.is_alive {
            return false;
        }

        self.health = self.health.saturating_sub(damage);

        if self.health == 0 {
            self.is_alive = false;
            self.respawn_timer = 0.0;
            true
        } else {
            false
        }
    }

    /// Get remaining respawn time
    pub fn respawn_time_remaining(&self) -> f32 {
        if self.is_alive {
            0.0
        } else {
            (player::RESPAWN_TIME - self.respawn_timer).max(0.0)
        }
    }

    /// Get distance to another position
    pub fn distance_to(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if point is within collision radius
    pub fn collides_with_point(&self, x: f32, y: f32, radius: f32) -> bool {
        self.distance_to(x, y) <= player::RADIUS + radius
    }

    /// Draw the player
    pub fn draw(&self, is_local: bool) {
        let color = if is_local { BLUE } else { RED };
        let arrow_color = if is_local { DARKBLUE } else { MAROON };

        if self.is_alive {
            // Draw player circle
            draw_circle(self.x, self.y, player::RADIUS, color);

            // Draw direction arrow
            let arrow_end_x = self.x + self.direction_x * player::ARROW_LENGTH;
            let arrow_end_y = self.y + self.direction_y * player::ARROW_LENGTH;
            draw_line(self.x, self.y, arrow_end_x, arrow_end_y, 2.0, arrow_color);

            // Draw health bar
            self.draw_health_bar();

            // Draw kill count for remote players
            if !is_local {
                self.draw_kill_count();
            }
        } else {
            // Draw ghost player
            let ghost_color = if is_local {
                Color::new(0.5, 0.5, 0.5, 0.5)
            } else {
                Color::new(0.8, 0.2, 0.2, 0.5)
            };

            draw_circle(self.x, self.y, player::RADIUS, ghost_color);
            self.draw_respawn_timer();
        }
    }

    /// Draw player health bar
    fn draw_health_bar(&self) {
        let bar_x = self.x - player::HEALTH_BAR_WIDTH / 2.0;
        let bar_y = self.y - 25.0;

        // Background
        draw_rectangle(
            bar_x,
            bar_y,
            player::HEALTH_BAR_WIDTH,
            player::HEALTH_BAR_HEIGHT,
            BLACK,
        );

        // Health fill
        let health_percentage = self.health as f32 / self.max_health as f32;
        let health_width = player::HEALTH_BAR_WIDTH * health_percentage;

        let health_color = match health_percentage {
            p if p > 0.6 => GREEN,
            p if p > 0.3 => YELLOW,
            _ => RED,
        };

        draw_rectangle(
            bar_x,
            bar_y,
            health_width,
            player::HEALTH_BAR_HEIGHT,
            health_color,
        );

        // Border
        draw_rectangle_lines(
            bar_x,
            bar_y,
            player::HEALTH_BAR_WIDTH,
            player::HEALTH_BAR_HEIGHT,
            1.0,
            WHITE,
        );

        // Health text
        let health_text = format!("{}/{}", self.health, self.max_health);
        let text_width = measure_text(&health_text, None, ui::TEXT_SIZE_NANO as u16, 1.0).width;
        draw_text(
            &health_text,
            bar_x + player::HEALTH_BAR_WIDTH / 2.0 - text_width / 2.0,
            bar_y + player::HEALTH_BAR_HEIGHT + ui::HEALTH_TEXT_OFFSET_Y,
            ui::HEALTH_TEXT_SIZE,
            WHITE,
        );
    }

    /// Draw kill count above player
    fn draw_kill_count(&self) {
        let kill_text = format!("K:{}", self.kills);
        let text_width = measure_text(&kill_text, None, ui::TEXT_SIZE_NANO as u16, 1.0).width;
        draw_text(
            &kill_text,
            self.x - text_width / 2.0,
            self.y - 35.0,
            ui::TEXT_SIZE_NANO,
            WHITE,
        );
    }

    /// Draw respawn timer
    fn draw_respawn_timer(&self) {
        let respawn_time_left = self.respawn_time_remaining();
        if respawn_time_left > 0.0 {
            let respawn_text = format!("Respawning: {respawn_time_left:.1}s");
            let text_width =
                measure_text(&respawn_text, None, ui::TEXT_SIZE_TINY as u16, 1.0).width;
            draw_text(
                &respawn_text,
                self.x - text_width / 2.0,
                self.y - 25.0,
                ui::TEXT_SIZE_TINY,
                WHITE,
            );
        }
    }
}
