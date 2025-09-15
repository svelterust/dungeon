const std = @import("std");
const rl = @import("raylib");
const Player = @import("Player.zig");
const Block = @import("Block.zig");
const Camera = @import("Camera.zig");
const utils = @import("utils.zig");

pub fn main() !void {
    // Initialize raylib
    var gpa = std.heap.GeneralPurposeAllocator(.{}).init;
    const allocator = gpa.allocator();
    rl.initWindow(utils.screenWidth, utils.screenHeight, "Dungeon");
    rl.setTargetFPS(60);
    defer rl.closeWindow();

    // Initialize textures
    const playerTexture = try rl.loadTexture("assets/player.png");
    defer playerTexture.unload();
    const blockTexture = try rl.loadTexture("assets/block.png");
    defer blockTexture.unload();

    // Initialize blocks
    const width = utils.screenWidth / Block.size;
    const height = utils.screenHeight / Block.size;
    var blocks: [width * height]Block = undefined;
    const sprites = [_]utils.Sprite{ Block.grass, Block.sand, Block.water };
    for (&blocks, 0..) |*block, i| {
        const x = i % width * Block.size;
        const y = i / width * Block.size;
        block.* = Block.init(@floatFromInt(x), @floatFromInt(y), blockTexture, sprites[(i / 8) % 3]);
    }

    // Initialize player
    var player = Player.init(utils.screenWidth / 2, utils.screenHeight / 2, playerTexture);
    defer player.deinit(allocator);

    // Initialize camera
    var camera = Camera.init(player.x, player.y);

    // Main game loop
    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        rl.clearBackground(.white);
        defer rl.endDrawing();
        rl.beginMode2D(camera.state);
        defer rl.endMode2D();

        // Objects
        for (blocks) |block| block.draw();
        try player.update(allocator);
        player.draw();
        camera.follow(&player);
    }
}
