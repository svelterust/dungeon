const rl = @import("raylib");
const utils = @import("utils.zig");
const Sprite = utils.Sprite;

// State
x: f32,
y: f32,
texture: rl.Texture2D,
sprite: Sprite,

// Constants
const Block = @This();
pub const size = 64;
pub const sand = Sprite{ 0, 0 };
pub const grass = Sprite{ 0, 1 };
pub const water = Sprite{ 0, 2 };

pub fn init(x: f32, y: f32, texture: rl.Texture2D, sprite: Sprite) Block {
    return .{ .x = x, .y = y, .texture = texture, .sprite = sprite };
}

pub fn draw(self: Block) void {
    utils.drawSprite(self.x, self.y, self.texture, self.sprite, .{});
}
