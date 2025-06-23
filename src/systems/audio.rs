//! Audio system for managing game sound effects

use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use std::collections::HashMap;

/// Audio system that manages all game sounds
pub struct AudioSystem {
    sounds: HashMap<SoundType, Sound>,
}

/// Types of sounds available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundType {
    PlayerShoot,
    BossShoot,
    Hit,
    Explosion,
    Join,
    PowerUp,
    Dash,
}

impl SoundType {
    /// Get the file path for this sound type
    fn file_path(&self) -> &'static str {
        match self {
            SoundType::PlayerShoot => "assets/bullet.wav",
            SoundType::BossShoot => "assets/boss_bullet.wav",
            SoundType::Hit => "assets/hit.wav",
            SoundType::Explosion => "assets/explosion.wav",
            SoundType::Join => "assets/join.wav",
            SoundType::PowerUp => "assets/powerup.wav",
            SoundType::Dash => "assets/dash.wav",
        }
    }
}

impl AudioSystem {
    /// Create a new audio system and preload all sounds
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut sounds = HashMap::new();

        // Preload all sound types
        let sound_types = [
            SoundType::PlayerShoot,
            SoundType::BossShoot,
            SoundType::Hit,
            SoundType::Explosion,
            SoundType::Join,
            SoundType::PowerUp,
            SoundType::Dash,
        ];

        for sound_type in sound_types {
            match load_sound(sound_type.file_path()).await {
                Ok(sound) => {
                    sounds.insert(sound_type, sound);
                }
                Err(e) => {
                    eprintln!("Failed to load sound {}: {}", sound_type.file_path(), e);
                    // Continue loading other sounds even if one fails
                }
            }
        }

        Ok(Self { sounds })
    }

    /// Play a sound effect
    pub fn play_sound(&self, sound_type: SoundType) {
        self.play_sound_with_params(sound_type, PlaySoundParams {
            looped: false,
            volume: 1.0,
        });
    }

    /// Play a sound effect with custom parameters
    pub fn play_sound_with_params(&self, sound_type: SoundType, params: PlaySoundParams) {
        if let Some(sound) = self.sounds.get(&sound_type) {
            play_sound(sound, params);
        } else {
            eprintln!("Sound not found: {:?}", sound_type);
        }
    }

    /// Play player shooting sound
    pub fn play_player_shoot(&self) {
        self.play_sound_with_params(
            SoundType::PlayerShoot,
            PlaySoundParams {
                looped: false,
                volume: 0.7,
            },
        );
    }

    /// Play boss shooting sound
    pub fn play_boss_shoot(&self) {
        self.play_sound_with_params(
            SoundType::BossShoot,
            PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );
    }

    /// Play hit sound when a bullet hits a target
    pub fn play_hit(&self) {
        self.play_sound_with_params(
            SoundType::Hit,
            PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
    }

    /// Play explosion sound when something dies
    pub fn play_explosion(&self) {
        self.play_sound_with_params(
            SoundType::Explosion,
            PlaySoundParams {
                looped: false,
                volume: 0.9,
            },
        );
    }

    /// Play join sound when a player joins
    pub fn play_join(&self) {
        self.play_sound_with_params(
            SoundType::Join,
            PlaySoundParams {
                looped: false,
                volume: 0.5,
            },
        );
    }

    /// Play power-up sound for boss abilities
    pub fn play_power_up(&self) {
        self.play_sound_with_params(
            SoundType::PowerUp,
            PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );
    }

    /// Play boss multi-shot sound (same as power-up)
    pub fn play_boss_multi_shot(&self) {
        self.play_power_up();
    }

    /// Play boss area attack sound (same as power-up)
    pub fn play_boss_area_attack(&self) {
        self.play_power_up();
    }

    /// Play boss shield sound (same as power-up)
    pub fn play_boss_shield(&self) {
        self.play_power_up();
    }

    /// Play boss dash sound
    pub fn play_boss_dash(&self) {
        self.play_sound_with_params(
            SoundType::Dash,
            PlaySoundParams {
                looped: false,
                volume: 0.9,
            },
        );
    }

    /// Play respawn sound (same as join)
    pub fn play_respawn(&self) {
        self.play_join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_type_file_paths() {
        assert_eq!(SoundType::PlayerShoot.file_path(), "assets/bullet.wav");
        assert_eq!(SoundType::BossShoot.file_path(), "assets/boss_bullet.wav");
        assert_eq!(SoundType::Hit.file_path(), "assets/hit.wav");
        assert_eq!(SoundType::Explosion.file_path(), "assets/explosion.wav");
        assert_eq!(SoundType::Join.file_path(), "assets/join.wav");
        assert_eq!(SoundType::PowerUp.file_path(), "assets/powerup.wav");
        assert_eq!(SoundType::Dash.file_path(), "assets/dash.wav");
    }
}