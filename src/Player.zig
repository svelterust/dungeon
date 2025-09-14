const std = @import("std");
const rl = @import("raylib");
const utils = @import("utils.zig");
const Bullet = @import("Bullet.zig");
const Sprite = utils.Sprite;

// State
x: f32,
y: f32,
flip: bool = false,
texture: rl.Texture,
bullets: std.ArrayList(Bullet) = std.ArrayList(Bullet).empty,
direction: enum { side, up, down } = .side,

// Constants
const Player = @This();
const size = 64;
const speed = 6;
const sprites = struct {
    const up = [_]Sprite{
        Sprite{ 0, 0 },
        Sprite{ 1, 0 },
        Sprite{ 2, 0 },
    };
    const down = [_]Sprite{
        Sprite{ 0, 1 },
        Sprite{ 1, 1 },
        Sprite{ 2, 1 },
    };
    const side = [_]Sprite{
        Sprite{ 0, 2 },
        Sprite{ 1, 2 },
    };
};

pub fn init(x: f32, y: f32) !Player {
    const texture = try rl.loadTexture("assets/player.png");
    return .{ .x = x, .y = y, .texture = texture };
}

pub fn draw(self: *Player) void {
    // Player
    const playerSprite = switch (self.direction) {
        .up => sprites.up[0],
        .down => sprites.down[0],
        .side => sprites.side[0],
    };
    utils.drawSprite(self.x, self.y, self.texture, playerSprite, .{ .flip = self.flip });

    // Bullets
    for (self.bullets.items) |*bullet| bullet.draw();
}

pub fn update(self: *Player, allocator: std.mem.Allocator) !void {
    // Player
    if (rl.isKeyDown(.e)) {
        self.y -= speed;
        if (!rl.isKeyDown(.d)) self.direction = .up;
    }
    if (rl.isKeyDown(.d)) {
        self.y += speed;
        if (!rl.isKeyDown(.e)) self.direction = .down;
    }
    if (rl.isKeyDown(.s)) {
        self.x -= speed;
        self.direction = .side;
        if (!rl.isKeyDown(.f)) self.flip = true;
    }
    if (rl.isKeyDown(.f)) {
        self.x += speed;
        self.direction = .side;
        if (!rl.isKeyDown(.s)) self.flip = false;
    }

    // Bullets
    if (rl.isMouseButtonPressed(.left)) {
        // Get angle
        const mouse_x: f32 = @floatFromInt(rl.getMouseX());
        const mouse_y: f32 = @floatFromInt(rl.getMouseY());
        const angle = std.math.atan2(mouse_y - self.y, mouse_x - self.x);
        try self.bullets.append(allocator, Bullet.init(self.x + size / 2, self.y + size / 2, angle));
    }
    for (self.bullets.items) |*bullet| bullet.update();
}

pub fn deinit(self: *Player, allocator: std.mem.Allocator) void {
    rl.unloadTexture(self.texture);
    self.bullets.deinit(allocator);
}
