const std = @import("std");
const rl = @import("raylib");
const Player = @import("Player.zig");
const Block = @import("Block.zig");

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

    // Initialize textures
    const playerTexture = try rl.loadTexture("assets/player.png");
    defer playerTexture.unload();
    const blockTexture = try rl.loadTexture("assets/block.png");
    defer blockTexture.unload();

    // Initialize blocks
    const width = screenWidth / Block.size;
    const height = screenHeight / Block.size;
    var blocks: [width * height]Block = undefined;
    for (&blocks, 0..) |*block, i| {
        const x = i % width * Block.size;
        const y = i / width * Block.size;
        block.* = Block.init(@floatFromInt(x), @floatFromInt(y), blockTexture);
    }

    // Initialize player
    var player = Player.init(screenWidth / 2, screenHeight / 2, playerTexture);
    defer player.deinit(allocator);

    // Main game loop
    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        rl.clearBackground(.white);
        defer rl.endDrawing();

        // Blocks
        for (blocks) |block| block.draw();

        // Player
        try player.update(allocator);
        player.draw();
    }
}
