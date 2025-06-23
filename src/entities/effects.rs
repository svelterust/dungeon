//! Visual effects entities for area attacks and damage indicators

use crate::constants::{alpha, area_attack, damage_indicator};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaAttack {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
    pub max_time: f32,
}

impl AreaAttack {
    /// Create a new area attack at the specified position
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            timer: 0.0,
            max_time: area_attack::DURATION,
        }
    }

    /// Update the area attack, return true if should be removed
    pub fn update(&mut self, dt: f32) -> bool {
        self.timer += dt;
        self.timer >= self.max_time
    }

    /// Get the progress of the area attack (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.timer / self.max_time).clamp(0.0, 1.0)
    }

    /// Check if a point is within the damage radius
    pub fn affects_point(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= area_attack::MAX_RADIUS
    }

    /// Get damage dealt by this area attack
    pub fn damage(&self) -> u32 {
        area_attack::DAMAGE
    }

    /// Draw the area attack effect
    pub fn draw(&self) {
        let progress = self.progress();
        let radius = area_attack::MAX_RADIUS * (1.0 - progress);
        let alpha = alpha::AREA_ATTACK * (1.0 - progress);

        // Draw expanding circle
        draw_circle(self.x, self.y, radius, Color::new(1.0, 0.3, 0.0, alpha));

        draw_circle_lines(
            self.x,
            self.y,
            radius,
            3.0,
            Color::new(1.0, 0.5, 0.0, alpha),
        );

        // Draw warning at center during first half of animation
        if progress < area_attack::WARNING_THRESHOLD {
            let warning_alpha = (area_attack::WARNING_THRESHOLD - progress) * 2.0;
            draw_circle(
                self.x,
                self.y,
                area_attack::WARNING_RADIUS,
                Color::new(1.0, 0.0, 0.0, warning_alpha),
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageIndicator {
    pub x: f32,
    pub y: f32,
    pub damage: u32,
    pub timer: f32,
    pub max_time: f32,
    pub from_player: bool,
}

impl DamageIndicator {
    /// Create a new damage indicator
    pub fn new(x: f32, y: f32, damage: u32, from_player: bool) -> Self {
        Self {
            x,
            y,
            damage,
            timer: 0.0,
            max_time: damage_indicator::DURATION,
            from_player,
        }
    }

    /// Update the damage indicator, return true if should be removed
    pub fn update(&mut self, dt: f32) -> bool {
        self.timer += dt;
        self.y -= damage_indicator::FLOAT_SPEED * dt; // Float upward
        self.timer >= self.max_time
    }

    /// Get the progress of the damage indicator (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.timer / self.max_time).clamp(0.0, 1.0)
    }

    /// Draw the damage indicator
    pub fn draw(&self) {
        let progress = self.progress();
        let alpha = alpha::DAMAGE_FADE * (1.0 - progress);

        let color = if self.from_player {
            Color::new(1.0, 0.4, 0.0, alpha) // Orange for PvP damage
        } else {
            Color::new(1.0, 0.0, 0.0, alpha) // Red for boss damage
        };

        let damage_text = format!("-{}", self.damage);
        let text_width =
            measure_text(&damage_text, None, damage_indicator::TEXT_SIZE as u16, 1.0).width;

        draw_text(
            &damage_text,
            self.x - text_width / 2.0,
            self.y,
            damage_indicator::TEXT_SIZE,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_attack_creation() {
        let attack = AreaAttack::new(100.0, 200.0);
        assert_eq!(attack.x, 100.0);
        assert_eq!(attack.y, 200.0);
        assert_eq!(attack.timer, 0.0);
        assert_eq!(attack.max_time, area_attack::DURATION);
    }

    #[test]
    fn test_area_attack_affects_point() {
        let attack = AreaAttack::new(100.0, 100.0);
        assert!(attack.affects_point(150.0, 150.0)); // Within radius
        assert!(!attack.affects_point(300.0, 300.0)); // Outside radius
    }

    #[test]
    fn test_damage_indicator_progress() {
        let mut indicator = DamageIndicator::new(0.0, 0.0, 10, false);
        assert_eq!(indicator.progress(), 0.0);

        indicator.timer = indicator.max_time / 2.0;
        assert_eq!(indicator.progress(), 0.5);

        indicator.timer = indicator.max_time;
        assert_eq!(indicator.progress(), 1.0);
    }
}
