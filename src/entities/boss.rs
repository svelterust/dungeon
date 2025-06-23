//! Boss entity with AI behavior and special abilities

use crate::constants::{boss, ui};
use crate::entities::Player;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Create a new boss at screen center
    pub fn new() -> Self {
        let x = screen_width() / 2.0;
        let y = boss::SPAWN_Y;

        Self {
            x,
            y,
            health: boss::MAX_HEALTH,
            max_health: boss::MAX_HEALTH,
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

    /// Respawn the boss with full health
    pub fn respawn(&mut self) {
        self.x = screen_width() / 2.0;
        self.y = boss::SPAWN_Y;
        self.health = self.max_health;
        self.alive = true;
        self.reset_all_timers();
    }

    /// Reset all boss timers
    fn reset_all_timers(&mut self) {
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

    /// Update boss AI and behavior
    pub fn update(&mut self, dt: f32, players: &[Player]) {
        if !self.alive {
            self.respawn_timer += dt;
            return;
        }

        self.update_timers(dt);
        self.update_movement(dt, players);
    }

    /// Update all boss timers
    fn update_timers(&mut self, dt: f32) {
        self.shoot_timer += dt;
        self.move_timer += dt;
        self.power_timer += dt;
        self.dash_timer += dt;

        // Update shield timer
        if self.shield_active {
            self.shield_timer += dt;
            if self.shield_timer >= boss::SHIELD_DURATION {
                self.shield_active = false;
                self.shield_timer = 0.0;
            }
        }
    }

    /// Update boss movement and dashing
    fn update_movement(&mut self, dt: f32, players: &[Player]) {
        if self.is_dashing {
            self.update_dash(dt);
            return;
        }

        self.update_target_selection(players);
        self.move_towards_target(dt);
    }

    /// Update dash movement
    fn update_dash(&mut self, dt: f32) {
        let dx = self.dash_target_x - self.x;
        let dy = self.dash_target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > boss::DASH_STOP_DISTANCE {
            let speed = boss::DASH_SPEED * dt;
            self.x += (dx / distance) * speed;
            self.y += (dy / distance) * speed;
        } else {
            self.is_dashing = false;
            self.dash_timer = 0.0;
        }
    }

    /// Update target selection for AI movement
    fn update_target_selection(&mut self, players: &[Player]) {
        if self.move_timer >= boss::MOVE_INTERVAL {
            if let Some(nearest_player) = self.find_nearest_player_to_boss(players) {
                self.set_random_target_near_player(&nearest_player);
            }
            self.move_timer = 0.0;
        }
    }

    /// Find the nearest alive player
    fn find_nearest_player_to_boss(&self, players: &[Player]) -> Option<Player> {
        players
            .iter()
            .filter(|p| p.is_alive)
            .min_by_key(|p| {
                let dx = p.x - self.x;
                let dy = p.y - self.y;
                (dx * dx + dy * dy) as i32
            })
            .cloned()
    }

    /// Set a random target position near the given player
    fn set_random_target_near_player(&mut self, player: &Player) {
        self.target_x = player.x + rand::gen_range(-boss::MOVEMENT_VARIANCE, boss::MOVEMENT_VARIANCE);
        self.target_y = player.y + rand::gen_range(-boss::MOVEMENT_VARIANCE, boss::MOVEMENT_VARIANCE);
        self.clamp_target_to_screen();
    }

    /// Keep boss target within screen bounds
    fn clamp_target_to_screen(&mut self) {
        let min_pos = boss::MIN_DISTANCE_FROM_EDGE;
        let max_x = screen_width() - boss::MIN_DISTANCE_FROM_EDGE;
        let max_y = screen_height() - boss::MIN_DISTANCE_FROM_EDGE;

        self.target_x = self.target_x.clamp(min_pos, max_x);
        self.target_y = self.target_y.clamp(min_pos, max_y);
    }

    /// Move towards current target
    fn move_towards_target(&mut self, dt: f32) {
        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 5.0 {
            let speed = boss::SPEED * dt;
            self.x += (dx / distance) * speed;
            self.y += (dy / distance) * speed;
        }
    }

    /// Check if boss should shoot
    pub fn should_shoot(&self) -> bool {
        self.alive && self.shoot_timer >= boss::SHOOT_INTERVAL
    }

    /// Reset shoot timer
    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = 0.0;
    }

    /// Check if boss should use a power ability
    pub fn should_use_power(&self) -> bool {
        self.alive && self.power_timer >= boss::POWER_INTERVAL && !self.is_dashing
    }

    /// Check if boss should dash
    pub fn should_dash(&self) -> bool {
        self.alive && self.dash_timer >= boss::DASH_INTERVAL && !self.is_dashing
    }

    /// Check if boss should respawn
    pub fn should_respawn(&self) -> bool {
        !self.alive && self.respawn_timer >= boss::RESPAWN_TIME
    }

    /// Take damage and return whether boss died
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if !self.alive || self.shield_active {
            return false;
        }

        if self.health <= damage {
            self.health = 0;
            self.alive = false;
            self.respawn_timer = 0.0;
            true
        } else {
            self.health -= damage;
            false
        }
    }

    /// Activate shield ability
    pub fn activate_shield(&mut self) {
        self.shield_active = true;
        self.shield_timer = 0.0;
    }

    /// Start dash ability towards target
    pub fn start_dash(&mut self, target_x: f32, target_y: f32) {
        self.is_dashing = true;
        self.dash_target_x = target_x;
        self.dash_target_y = target_y;
        self.dash_timer = 0.0;
    }

    /// Reset power timer
    pub fn reset_power_timer(&mut self) {
        self.power_timer = 0.0;
    }

    /// Get distance to a point
    pub fn distance_to(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if point collides with boss
    pub fn collides_with_point(&self, x: f32, y: f32, radius: f32) -> bool {
        self.distance_to(x, y) <= boss::RADIUS + radius
    }

    /// Get power warning time remaining
    pub fn power_warning_time(&self) -> f32 {
        let time_left = boss::POWER_INTERVAL - self.power_timer;
        if time_left <= ui::WARNING_DISPLAY_TIME && time_left > 0.0 {
            time_left
        } else {
            0.0
        }
    }

    /// Get dash warning time remaining
    pub fn dash_warning_time(&self) -> f32 {
        let time_left = boss::DASH_INTERVAL - self.dash_timer;
        if time_left <= ui::WARNING_DISPLAY_TIME && time_left > 0.0 {
            time_left
        } else {
            0.0
        }
    }

    /// Get respawn time remaining
    pub fn respawn_time_remaining(&self) -> f32 {
        if self.alive {
            0.0
        } else {
            (boss::RESPAWN_TIME - self.respawn_timer).max(0.0)
        }
    }

    /// Draw the boss
    pub fn draw(&self) {
        if self.alive {
            self.draw_alive_boss();
        } else {
            self.draw_respawn_timer();
        }
    }

    /// Draw the alive boss with effects
    fn draw_alive_boss(&self) {
        // Draw shield effect
        if self.shield_active {
            draw_circle(self.x, self.y, boss::SHIELD_RADIUS, Color::new(0.0, 0.5, 1.0, 0.3));
            draw_circle_lines(self.x, self.y, boss::SHIELD_RADIUS, 3.0, BLUE);
        }

        // Draw dash effect
        if self.is_dashing {
            draw_circle(self.x, self.y, boss::DASH_EFFECT_RADIUS, Color::new(1.0, 0.5, 0.0, 0.4));
        }

        // Draw boss body
        draw_circle(self.x, self.y, boss::RADIUS, MAROON);
        draw_circle(self.x, self.y, boss::INNER_RADIUS, RED);

        // Draw health bar
        self.draw_health_bar();

        // Draw warnings
        self.draw_warnings();
    }

    /// Draw boss health bar
    fn draw_health_bar(&self) {
        let bar_x = self.x - boss::HEALTH_BAR_WIDTH / 2.0;
        let bar_y = self.y - boss::HEALTH_BAR_OFFSET_Y;

        // Background
        draw_rectangle(bar_x, bar_y, boss::HEALTH_BAR_WIDTH, boss::HEALTH_BAR_HEIGHT, BLACK);

        // Health fill
        let health_percentage = self.health as f32 / self.max_health as f32;
        let health_width = boss::HEALTH_BAR_WIDTH * health_percentage;
        draw_rectangle(bar_x, bar_y, health_width, boss::HEALTH_BAR_HEIGHT, RED);

        // Border
        draw_rectangle_lines(bar_x, bar_y, boss::HEALTH_BAR_WIDTH, boss::HEALTH_BAR_HEIGHT, 1.0, WHITE);
    }

    /// Draw power and dash warnings
    fn draw_warnings(&self) {
        // Power warning
        if self.power_warning_time() > 0.0 {
            let warning_text = "BOSS POWER INCOMING!";
            let text_width = measure_text(warning_text, None, ui::TEXT_SIZE_MEDIUM as u16, 1.0).width;
            draw_text(
                warning_text,
                screen_width() / 2.0 - text_width / 2.0,
                50.0,
                ui::TEXT_SIZE_MEDIUM,
                ORANGE,
            );
        }

        // Dash warning
        if self.dash_warning_time() > 0.0 {
            let dash_text = "BOSS DASH INCOMING!";
            let text_width = measure_text(dash_text, None, ui::TEXT_SIZE_SMALL as u16, 1.0).width;
            draw_text(
                dash_text,

                screen_width() / 2.0 - text_width / 2.0,
                70.0,
                ui::TEXT_SIZE_SMALL,
                YELLOW,
            );
        }
    }

    /// Draw respawn timer
    fn draw_respawn_timer(&self) {
        let respawn_time_left = self.respawn_time_remaining();
        if respawn_time_left > 0.0 {
            let respawn_text = format!("Boss respawning in: {respawn_time_left:.1}s");
            let text_width = measure_text(&respawn_text, None, ui::TEXT_SIZE_LARGE as u16, 1.0).width;
            draw_text(
                &respawn_text,
                screen_width() / 2.0 - text_width / 2.0,
                100.0,
                ui::TEXT_SIZE_LARGE,
                DARKGREEN,
            );
        }
    }
}

impl Default for Boss {
    fn default() -> Self {
        Self::new()
    }
}