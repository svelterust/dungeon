const rl = @import("raylib");

// Constants
pub const screenWidth = 1280;
pub const screenHeight = 720;

pub const Sprite = struct { i32, i32 };

pub const Options = struct {
    flip: bool = false,
};

pub fn drawSprite(x: f32, y: f32, texture: rl.Texture2D, sprite: Sprite, options: Options) void {
    const size = 64;
    const width: f32 = if (options.flip) -size else size;
    const source = rl.Rectangle{
        .x = @floatFromInt(sprite[0] * size),
        .y = @floatFromInt(sprite[1] * size),
        .width = width,
        .height = size,
    };
    const dest = rl.Rectangle{ .x = x, .y = y, .width = size, .height = size };
    rl.drawTexturePro(texture, source, dest, .{ .x = 0, .y = 0 }, 0.0, .white);
}
