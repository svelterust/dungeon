const rl = @import("raylib");

const Player = struct {
    x: f32,
    y: f32,
};

pub fn main() !void {
    // Initialize raylib
    const screenWidth = 1280;
    const screenHeight = 720;
    rl.initWindow(screenWidth, screenHeight, "Project");
    defer rl.closeWindow();
    rl.setTargetFPS(60);

    // Main game loop
    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        defer rl.endDrawing();
        rl.clearBackground(.white);
        rl.drawText("Project!", screenHeight / 2, screenHeight / 2, 36, .black);
    }
}
