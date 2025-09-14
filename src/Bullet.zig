const std = @import("std");
const rl = @import("raylib");

// State
x: f32,
y: f32,
direction: f32,
rotation: f32 = 0,

// Constants
const Bullet = @This();
const speed = 10;
const size = 16;

pub fn init(x: f32, y: f32, direction: f32) Bullet {
    return Bullet{
        .x = x,
        .y = y,
        .direction = direction,
    };
}

pub fn draw(self: *Bullet) void {
    rl.drawRectanglePro(
        .{ .x = self.x, .y = self.y, .width = size, .height = size },
        .{ .x = size / 2, .y = size / 2 },
        self.rotation,
        .gray,
    );
}

pub fn update(self: *Bullet) void {
    self.rotation += 3;
    if (self.rotation > 360) self.rotation = 0;
    self.x += std.math.cos(self.direction) * speed;
    self.y += std.math.sin(self.direction) * speed;
}
