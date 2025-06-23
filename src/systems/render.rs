//! Rendering system for UI and game elements

use crate::constants::{ui, network};
use crate::entities::{Player, Boss, Bullet, AreaAttack, DamageIndicator};
use macroquad::prelude::*;

/// Handles all rendering operations for the game
pub struct RenderSystem;

impl RenderSystem {
    /// Draw the complete game UI
    pub fn draw_ui(
        local_player: &Player,
        remote_players: &[Player],
        boss: &Boss,
    ) {
        Self::draw_player_count(local_player, remote_players);
        Self::draw_kill_count(local_player);
        Self::draw_leaderboard(local_player, remote_players);
        Self::draw_controls();
    }

    /// Draw player count in top left
    fn draw_player_count(local_player: &Player, remote_players: &[Player]) {
        let total_players = 1 + remote_players.len();
        let player_text = format!("Players Connected: {}", total_players);
        draw_text(&player_text, ui::MARGIN, 30.0, ui::LINE_HEIGHT, BLACK);
    }

    /// Draw local player kill count
    fn draw_kill_count(local_player: &Player) {
        let kills_text = format!("Your Kills: {}", local_player.kills);
        draw_text(&kills_text, ui::MARGIN, 50.0, ui::TEXT_SIZE_SMALL, DARKBLUE);
    }

    /// Draw leaderboard
    fn draw_leaderboard(local_player: &Player, remote_players: &[Player]) {
        draw_text("LEADERBOARD", ui::MARGIN, 90.0, ui::TEXT_SIZE_TINY, DARKGRAY);
        
        // Collect all players with their kills
        let mut players_with_kills = vec![(local_player.id, local_player.kills, "You".to_string())];
        for player in remote_players {
            players_with_kills.push((player.id, player.kills, format!("Player {}", player.id)));
        }
        
        // Sort by kills (descending)
        players_with_kills.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Draw top entries
        for (i, (_, kills, name)) in players_with_kills.iter().take(ui::LEADERBOARD_MAX_ENTRIES).enumerate() {
            let y_pos = 110.0 + (i as f32 * ui::SMALL_LINE_HEIGHT);
            let rank_text = format!("{}. {} - {} kills", i + 1, name, kills);
            let color = if name == "You" { DARKBLUE } else { DARKGRAY };
            draw_text(&rank_text, ui::MARGIN, y_pos, ui::TEXT_SIZE_MICRO, color);
        }
    }

    /// Draw control instructions
    fn draw_controls() {
        let controls = [
            ("ESC: Quit", screen_height() - 60.0),
            ("WASD/Arrow Keys: Move", screen_height() - 40.0),
            ("Space: Shoot", screen_height() - 20.0),
        ];

        for (text, y_pos) in controls {
            draw_text(text, ui::MARGIN, y_pos, ui::TEXT_SIZE_TINY, DARKGRAY);
        }

        // PvP info
        draw_text(
            "PvP: Players can shoot each other!",
            ui::MARGIN,
            screen_height() - 80.0,
            ui::TEXT_SIZE_MICRO,
            ORANGE,
        );
        draw_text(
            "Kill other players to climb the leaderboard!",
            ui::MARGIN,
            screen_height() - 100.0,
            ui::TEXT_SIZE_MICRO,
            GOLD,
        );
    }

    /// Draw all game entities
    pub fn draw_entities(
        local_player: &Player,
        remote_players: &[Player],
        boss: &Boss,
        bullets: &[Bullet],
        area_attacks: &[AreaAttack],
        damage_indicators: &[DamageIndicator],
    ) {
        // Draw boss
        boss.draw();

        // Draw local player
        local_player.draw(true);

        // Draw remote players
        for player in remote_players {
            player.draw(false);
        }

        // Draw area attacks
        for area_attack in area_attacks {
            area_attack.draw();
        }

        // Draw bullets
        for bullet in bullets {
            bullet.draw_with_owner_check(local_player.id);
        }

        // Draw damage indicators
        for indicator in damage_indicators {
            indicator.draw();
        }
    }

    /// Clear screen with background color
    pub fn clear_screen() {
        clear_background(WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_system_creation() {
        // RenderSystem is a unit struct, so just test that it exists
        let _system = RenderSystem;
    }
}