//! Bullet entity for projectiles

use crate::constants::{bullet, ui};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub lifetime: f32,
    pub owner_id: u32,
    pub is_boss_bullet: bool,
}

impl Bullet {
    /// Create a new player bullet
    pub fn new(x: f32, y: f32, direction_x: f32, direction_y: f32, owner_id: u32) -> Self {
        Self {
            x,
            y,
            velocity_x: direction_x * bullet::PLAYER_SPEED,
            velocity_y: direction_y * bullet::PLAYER_SPEED,
            lifetime: bullet::PLAYER_LIFETIME,
            owner_id,
            is_boss_bullet: false,
        }
    }

    /// Create a new boss bullet
    pub fn new_boss_bullet(x: f32, y: f32, direction_x: f32, direction_y: f32) -> Self {
        Self {
            x,
            y,
            velocity_x: direction_x * bullet::BOSS_SPEED,
            velocity_y: direction_y * bullet::BOSS_SPEED,
            lifetime: bullet::BOSS_LIFETIME,
            owner_id: 0, // Special ID for boss
            is_boss_bullet: true,
        }
    }

    /// Update bullet position and lifetime, return true if should be removed
    pub fn update(&mut self, dt: f32) -> bool {
        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;
        self.lifetime -= dt;

        // Remove if expired or off-screen
        self.lifetime <= 0.0 || self.is_off_screen()
    }

    /// Check if bullet is off screen
    fn is_off_screen(&self) -> bool {
        self.x < 0.0 || self.x > screen_width() || self.y < 0.0 || self.y > screen_height()
    }

    /// Get distance to a point
    pub fn distance_to(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check collision with a circular target
    pub fn collides_with(&self, x: f32, y: f32, radius: f32) -> bool {
        let bullet_radius = if self.is_boss_bullet {
            bullet::BOSS_RADIUS
        } else {
            bullet::RADIUS
        };
        self.distance_to(x, y) <= bullet_radius + radius
    }

    /// Get damage dealt by this bullet
    pub fn damage(&self) -> u32 {
        if self.is_boss_bullet {
            bullet::DAMAGE_BOSS_TO_PLAYER
        } else {
            bullet::DAMAGE_PLAYER
        }
    }

    /// Draw the bullet
    pub fn draw(&self) {
        if self.is_boss_bullet {
            draw_circle(self.x, self.y, bullet::BOSS_RADIUS, MAROON);
            draw_circle(self.x, self.y, bullet::BOSS_INNER_RADIUS, ORANGE);
        } else {
            let color = match self.owner_id {
                0 => DARKPURPLE, // Should not happen, but fallback
                id if id == self.owner_id => DARKBLUE, // This logic needs fixing in context
                _ => DARKPURPLE, // Other players
            };
            draw_circle(self.x, self.y, bullet::RADIUS, color);
        }
    }

    /// Draw bullet with specific color for local player identification
    pub fn draw_with_owner_check(&self, local_player_id: u32) {
        if self.is_boss_bullet {
            draw_circle(self.x, self.y, bullet::BOSS_RADIUS, MAROON);
            draw_circle(self.x, self.y, bullet::BOSS_INNER_RADIUS, ORANGE);
        } else {
            let color = if self.owner_id == local_player_id {
                DARKBLUE
            } else {
                DARKPURPLE
            };
            draw_circle(self.x, self.y, bullet::RADIUS, color);
        }
    }
}