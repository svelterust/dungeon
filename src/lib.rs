use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Move(u32, f32, f32), // player_id, x, y
    Join(u32),
    Leave(u32),
}
