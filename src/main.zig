const std = @import("std");
const rl = @import("raylib");
const Player = @import("Player.zig");

// Constants
const screenWidth = 1280;
const screenHeight = 720;

pub fn main() !void {
    // Initialize raylib
    var gpa = std.heap.GeneralPurposeAllocator(.{}).init;
    const allocator = gpa.allocator();
    rl.initWindow(screenWidth, screenHeight, "Dungeon");
    rl.setTargetFPS(60);
    defer rl.closeWindow();

    // Initialize objects
    var player = try Player.init(screenWidth / 2, screenHeight / 2);
    defer rl.unloadTexture(player.texture);

    // Main game loop
    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        rl.clearBackground(.white);
        defer rl.endDrawing();

        // Player
        try player.update(allocator);
        player.draw();
    }
}
