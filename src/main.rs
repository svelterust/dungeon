use anyhow::Result;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Payload {
    Move { x: f32, y: f32 },
}

#[macroquad::main("Dungeon")]
async fn main() -> Result<()> {
    loop {
        // Draw some objects
        clear_background(WHITE);
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 45.0, screen_height() - 45.0, 30.0, PURPLE);

        // Draw payload json
        let payload = Payload::Move { x: 10.0, y: 15.0 };
        let json = serde_json::to_string(&payload)?;
        draw_text(&json, 150.0, 300.0, 40.0, BLACK);
        next_frame().await
    }
}
