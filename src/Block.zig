const rl = @import("raylib");
const utils = @import("utils.zig");
const Sprite = utils.Sprite;

// State
x: f32,
y: f32,
texture: rl.Texture,

// Constants
pub const size = 64;
const Block = @This();
const sprites = struct {
    const grass = Sprite{ 0, 0 };
};

pub fn init(x: f32, y: f32, texture: rl.Texture) Block {
    return .{ .x = x, .y = y, .texture = texture };
}

pub fn draw(self: Block) void {
    utils.drawSprite(self.x, self.y, self.texture, sprites.grass, .{});
}
