const rl = @import("raylib");

// Constants
pub const screenWidth = 1280;
pub const screenHeight = 720;

pub const Sprite = struct { i32, i32 };

pub const Options = struct {
    flip: bool = false,
};

pub fn drawSprite(x: f32, y: f32, texture: rl.Texture2D, sprite: Sprite, options: Options) void {
    const sprite_width = 64;
    const sprite_height = 64;
    const width: f32 = if (options.flip) -sprite_width else sprite_width;
    const source = rl.Rectangle{
        .x = @floatFromInt(sprite[0] * sprite_width),
        .y = @floatFromInt(sprite[1] * sprite_height),
        .width = width,
        .height = sprite_height,
    };
    const dest = rl.Rectangle{ .x = x, .y = y, .width = sprite_width, .height = sprite_height };
    rl.drawTexturePro(texture, source, dest, .{ .x = 0, .y = 0 }, 0.0, .white);
}
