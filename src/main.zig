const std = @import("std");
const rl = @import("raylib");
const math = std.math;

const Sprite = struct {
    frame_x: i32,
    frame_y: i32,
};

fn drawSprite(x: f32, y: f32, texture: rl.Texture2D, sprite: Sprite) void {
    const sprite_width = 64;
    const sprite_height = 64;
    const source = rl.Rectangle{ .x = @floatFromInt(sprite.frame_x * sprite_width), .y = @floatFromInt(sprite.frame_y * sprite_height), .width = sprite_width, .height = sprite_height };
    const dest = rl.Rectangle{ .x = x, .y = y, .width = sprite_width, .height = sprite_height };
    rl.drawTexturePro(texture, source, dest, .{ .x = 0, .y = 0 }, 0.0, .white);
}

const Bullet = struct {
    x: f32,
    y: f32,
    size: f32,
    speed: f32,
    direction: f32,
    rotation: f32,

    fn init(x: f32, y: f32, direction: f32) Bullet {
        return Bullet{
            .x = x,
            .y = y,
            .size = 16,
            .speed = 10,
            .rotation = 0.0,
            .direction = direction,
        };
    }

    fn draw(self: *Bullet) void {
        rl.drawRectanglePro(.{
            .x = self.x,
            .y = self.y,
            .width = self.size,
            .height = self.size,
        }, .{ .x = self.size / 2, .y = self.size / 2 }, self.rotation, .blue);
    }

    fn update(self: *Bullet) void {
        self.rotation += 3;
        if (self.rotation > 360) self.rotation = 0;
        self.x += std.math.cos(self.direction) * self.speed;
        self.y += std.math.sin(self.direction) * self.speed;
    }
};

const Player = struct {
    x: f32,
    y: f32,
    speed: f32,
    texture: rl.Texture,
    bullets: std.ArrayList(Bullet),

    // Constants
    pub const size = 64;
    const path = "assets/player.png";
    const regular = Sprite{ .frame_x = 0, .frame_y = 0 };
    const walking = Sprite{ .frame_x = 1, .frame_y = 0 };

    fn init(x: f32, y: f32) !Player {
        const texture = try rl.loadTexture(path);
        const bullets = std.ArrayList(Bullet).empty;
        return .{ .x = x, .y = y, .speed = 7, .texture = texture, .bullets = bullets };
    }

    fn draw(self: *Player) void {
        // Player
        drawSprite(self.x, self.y, self.texture, regular);

        // Bullets
        for (self.bullets.items) |*bullet| bullet.draw();
    }

    fn update(self: *Player, allocator: std.mem.Allocator) !void {
        // Player
        if (rl.isKeyDown(.s)) self.x -= self.speed;
        if (rl.isKeyDown(.f)) self.x += self.speed;
        if (rl.isKeyDown(.e)) self.y -= self.speed;
        if (rl.isKeyDown(.d)) self.y += self.speed;
        if (rl.isKeyDown(.a)) self.y += self.speed;

        // Bullets
        if (rl.isMouseButtonPressed(.left)) {
            // Get angle
            const mouse_x: f32 = @floatFromInt(rl.getMouseX());
            const mouse_y: f32 = @floatFromInt(rl.getMouseY());
            const angle = math.atan2(mouse_y - self.y, mouse_x - self.x);
            try self.bullets.append(allocator, Bullet.init(
                self.x + Player.size / 2,
                self.y + Player.size / 2,
                angle,
            ));
        }
        for (self.bullets.items) |*bullet| bullet.update();
    }
};

// Constants
const screenWidth = 1280;
const screenHeight = 720;

pub fn main() !void {
    // Initialize raylib
    var gpa = std.heap.GeneralPurposeAllocator(.{}).init;
    const allocator = gpa.allocator();
    rl.initWindow(screenWidth, screenHeight, "Project");
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
