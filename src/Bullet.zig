const std = @import("std");
const rl = @import("raylib");

const Bullet = @This();

x: f32,
y: f32,
size: f32,
speed: f32,
direction: f32,
rotation: f32,

pub fn init(x: f32, y: f32, direction: f32) Bullet {
    return Bullet{
        .x = x,
        .y = y,
        .size = 16,
        .speed = 10,
        .rotation = 0.0,
        .direction = direction,
    };
}

pub fn draw(self: *Bullet) void {
    rl.drawRectanglePro(
        .{ .x = self.x, .y = self.y, .width = self.size, .height = self.size },
        .{ .x = self.size / 2, .y = self.size / 2 },
        self.rotation,
        .gray,
    );
}

pub fn update(self: *Bullet) void {
    self.rotation += 3;
    if (self.rotation > 360) self.rotation = 0;
    self.x += std.math.cos(self.direction) * self.speed;
    self.y += std.math.sin(self.direction) * self.speed;
}
