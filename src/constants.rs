//! Game constants and configuration values

/// Player configuration
pub mod player {
    pub const RADIUS: f32 = 15.0;
    pub const SPEED: f32 = 200.0;
    pub const MAX_HEALTH: u32 = 100;
    pub const RESPAWN_TIME: f32 = 5.0;
    pub const ARROW_LENGTH: f32 = 25.0;
    pub const HEALTH_BAR_WIDTH: f32 = 40.0;
    pub const HEALTH_BAR_HEIGHT: f32 = 4.0;
    pub const COLLISION_RADIUS: f32 = 18.0; // Player radius + bullet radius
}

/// Boss configuration
pub mod boss {
    pub const RADIUS: f32 = 50.0;
    pub const INNER_RADIUS: f32 = 45.0;
    pub const SHIELD_RADIUS: f32 = 60.0;
    pub const DASH_EFFECT_RADIUS: f32 = 55.0;
    pub const MAX_HEALTH: u32 = 100;
    pub const SPEED: f32 = 50.0;
    pub const DASH_SPEED: f32 = 400.0;
    pub const SHOOT_INTERVAL: f32 = 1.5;
    pub const MOVE_INTERVAL: f32 = 2.0;
    pub const POWER_INTERVAL: f32 = 8.0;
    pub const DASH_INTERVAL: f32 = 12.0;
    pub const SHIELD_DURATION: f32 = 3.0;
    pub const RESPAWN_TIME: f32 = 5.0;
    pub const SPAWN_Y: f32 = 100.0;
    pub const MOVEMENT_VARIANCE: f32 = 100.0;
    pub const MIN_DISTANCE_FROM_EDGE: f32 = 50.0;
    pub const COLLISION_RADIUS: f32 = 53.0; // Boss radius + bullet radius
    pub const DASH_STOP_DISTANCE: f32 = 10.0;
    pub const HEALTH_BAR_WIDTH: f32 = 100.0;
    pub const HEALTH_BAR_HEIGHT: f32 = 8.0;
    pub const HEALTH_BAR_OFFSET_Y: f32 = 70.0;
}

/// Bullet configuration
pub mod bullet {
    pub const RADIUS: f32 = 3.0;
    pub const BOSS_RADIUS: f32 = 5.0;
    pub const BOSS_INNER_RADIUS: f32 = 3.0;
    pub const PLAYER_SPEED: f32 = 400.0;
    pub const BOSS_SPEED: f32 = 300.0;
    pub const PLAYER_LIFETIME: f32 = 3.0;
    pub const BOSS_LIFETIME: f32 = 4.0;
    pub const DAMAGE_PLAYER: u32 = 15;
    pub const DAMAGE_BOSS: u32 = 10;
    pub const DAMAGE_BOSS_TO_PLAYER: u32 = 10;
}

/// Area attack configuration
pub mod area_attack {
    pub const MAX_RADIUS: f32 = 100.0;
    pub const DURATION: f32 = 1.0;
    pub const DAMAGE: u32 = 20;
    pub const WARNING_RADIUS: f32 = 10.0;
    pub const WARNING_THRESHOLD: f32 = 0.5;
}

/// Damage indicator configuration
pub mod damage_indicator {
    pub const DURATION: f32 = 1.5;
    pub const FLOAT_SPEED: f32 = 30.0;
    pub const TEXT_SIZE: f32 = 18.0;
}

/// Boss multi-shot configuration
pub mod multi_shot {
    pub const BULLET_COUNT: i32 = 5;
    pub const SPREAD_ANGLE: f32 = 0.2;
    pub const ANGLE_RANGE: std::ops::RangeInclusive<i32> = -2..=2;
}

/// UI configuration
pub mod ui {
    pub const TEXT_SIZE_LARGE: f32 = 24.0;
    pub const TEXT_SIZE_MEDIUM: f32 = 20.0;
    pub const TEXT_SIZE_SMALL: f32 = 18.0;
    pub const TEXT_SIZE_TINY: f32 = 16.0;
    pub const TEXT_SIZE_MICRO: f32 = 14.0;
    pub const TEXT_SIZE_NANO: f32 = 12.0;
    
    pub const MARGIN: f32 = 10.0;
    pub const LINE_HEIGHT: f32 = 20.0;
    pub const SMALL_LINE_HEIGHT: f32 = 16.0;
    
    pub const LEADERBOARD_MAX_ENTRIES: usize = 5;
    pub const WARNING_DISPLAY_TIME: f32 = 2.0;
    
    pub const HEALTH_TEXT_SIZE: f32 = 12.0;
    pub const HEALTH_TEXT_OFFSET_Y: f32 = 12.0;
}

/// Network configuration
pub mod network {
    pub const BUFFER_SIZE: usize = 1024;
    pub const MAX_MESSAGES_PER_FRAME: usize = 100;
    pub const DEFAULT_DEBUG_ADDRESS: &str = "0.0.0.0:9000";
    pub const DEFAULT_PROD_ADDRESS: &str = "dungeon.svelterust.com:9000";
    pub const DEFAULT_PORT: u16 = 9000;
}

/// Color alpha values for transparency effects
pub mod alpha {
    pub const SHIELD_EFFECT: f32 = 0.3;
    pub const DASH_EFFECT: f32 = 0.4;
    pub const AREA_ATTACK: f32 = 0.3;
    pub const GHOST_PLAYER: f32 = 0.5;
    pub const DAMAGE_FADE: f32 = 1.0; // Will be reduced by progress
}

/// Game boundaries and positioning
pub mod bounds {
    pub const PLAYER_MIN_DISTANCE_FROM_EDGE: f32 = 15.0;
    pub const SCREEN_EDGE_BUFFER: f32 = 50.0;
}