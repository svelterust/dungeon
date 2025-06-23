use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Move(u32, f32, f32), // player_id, x, y
    Join(u32),
    Leave(u32),
    Shoot(u32, f32, f32, f32, f32), // player_id, x, y, direction_x, direction_y
    BossShoot(f32, f32, f32, f32),  // x, y, direction_x, direction_y
    PlayerHit(u32, u32),            // player_id, new_health
    BossHit(u32),                   // new_boss_health
    BossSpawn(f32, f32),            // x, y
    BossDead,
    BossMultiShoot(f32, f32, Vec<(f32, f32)>), // x, y, directions
    BossDash(f32, f32),                        // target_x, target_y
    BossAreaAttack(f32, f32),                  // center_x, center_y
    BossShield(bool),                          // shield_active
    PlayerRespawn(u32, f32, f32),              // player_id, x, y
    PlayerDirection(u32, f32, f32),            // player_id, direction_x, direction_y
    PlayerKill(u32, u32),                      // killer_id, victim_id
}
