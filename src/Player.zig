const std = @import("std");
const rl = @import("raylib");
const utils = @import("utils.zig");
const Bullet = @import("Bullet.zig");

const Player = @This();

x: f32,
y: f32,
texture: rl.Texture,
bullets: std.ArrayList(Bullet),
direction: enum { left, right },

// Constants
pub const size = 64;
const speed = 6;
const path = "assets/player.png";
const regular = utils.Sprite{ .frame_x = 0, .frame_y = 0 };
const walking = utils.Sprite{ .frame_x = 1, .frame_y = 0 };

pub fn init(x: f32, y: f32) !Player {
    const texture = try rl.loadTexture(path);
    const bullets = std.ArrayList(Bullet).empty;
    return .{
        .x = x,
        .y = y,
        .texture = texture,
        .bullets = bullets,
        .direction = .right,
    };
}

pub fn draw(self: *Player) void {
    // Player
    const flip = self.direction == .left;
    utils.drawSprite(self.x, self.y, self.texture, regular, .{ .flip = flip });

    // Bullets
    for (self.bullets.items) |*bullet| bullet.draw();
}

pub fn update(self: *Player, allocator: std.mem.Allocator) !void {
    // Player
    if (rl.isKeyDown(.s)) {
        self.x -= speed;
        if (!rl.isKeyDown(.f)) self.direction = .left;
    }
    if (rl.isKeyDown(.f)) {
        self.x += speed;
        if (!rl.isKeyDown(.s)) self.direction = .right;
    }
    if (rl.isKeyDown(.e)) self.y -= speed;
    if (rl.isKeyDown(.d)) self.y += speed;
    if (rl.isKeyDown(.a)) self.y += speed;

    // Bullets
    if (rl.isMouseButtonPressed(.left)) {
        // Get angle
        const mouse_x: f32 = @floatFromInt(rl.getMouseX());
        const mouse_y: f32 = @floatFromInt(rl.getMouseY());
        const angle = std.math.atan2(mouse_y - self.y, mouse_x - self.x);
        try self.bullets.append(allocator, Bullet.init(
            self.x + Player.size / 2,
            self.y + Player.size / 2,
            angle,
        ));
    }
    for (self.bullets.items) |*bullet| bullet.update();
}
