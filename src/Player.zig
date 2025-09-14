const std = @import("std");
const rl = @import("raylib");
const utils = @import("utils.zig");
const Bullet = @import("Bullet.zig");
const Sprite = utils.Sprite;

// State
x: f32,
y: f32,
texture: rl.Texture,
flip: bool = false,
moving: bool = false,
timer: u32 = 0,
direction: enum { side, up, down } = .side,
bullets: std.ArrayList(Bullet) = std.ArrayList(Bullet).empty,

// Constants
const Player = @This();
const size = 64;
const speed = 6;
const animationSpeed = 10;
const sprites = struct {
    const up = Sprite{ 0, 0 };
    const down = Sprite{ 0, 1 };
    const side = Sprite{ 0, 2 };
    const movingUp = [_]Sprite{
        Sprite{ 1, 0 },
        Sprite{ 2, 0 },
    };
    const movingDown = [_]Sprite{
        Sprite{ 1, 1 },
        Sprite{ 2, 1 },
    };
    const movingSide = [_]Sprite{
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
    const frame = self.timer / animationSpeed;
    const playerSprite = switch (self.direction) {
        .up => if (self.moving) sprites.movingUp[frame % sprites.movingUp.len] else sprites.up,
        .down => if (self.moving) sprites.movingDown[frame % sprites.movingDown.len] else sprites.down,
        .side => if (self.moving) sprites.movingSide[frame % sprites.movingSide.len] else sprites.side,
    };
    utils.drawSprite(self.x, self.y, self.texture, playerSprite, .{ .flip = self.flip });

    // Bullets
    for (self.bullets.items) |*bullet| bullet.draw();
}

pub fn update(self: *Player, allocator: std.mem.Allocator) !void {
    // Player
    self.moving = false;
    const up, const down, const left, const right = .{
        rl.isKeyDown(.e),
        rl.isKeyDown(.d),
        rl.isKeyDown(.s),
        rl.isKeyDown(.f),
    };
    if (up) {
        self.y -= speed;
        if (!down) self.direction = .up;
    }
    if (down) {
        self.y += speed;
        if (!up) self.direction = .down;
    }
    if (left) {
        self.x -= speed;
        self.direction = .side;
        if (!right) self.flip = true;
    }
    if (right) {
        self.x += speed;
        self.direction = .side;
        if (!left) self.flip = false;
    }
    if (up or down or left or right) {
        self.timer += 1;
        self.moving = true;
    } else self.timer = 0;

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
