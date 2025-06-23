use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Move(f32, f32),
    Join(u32),
    Leave(u32),
}
