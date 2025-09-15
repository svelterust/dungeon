const rl = @import("raylib");
const Player = @import("Player.zig");
const utils = @import("utils.zig");

// State
state: rl.Camera2D,

// Constants
const Camera = @This();
const speed = 7;

pub fn init(x: f32, y: f32) Camera {
    return .{
        .state = rl.Camera2D{
            .offset = rl.Vector2{ .x = utils.screenWidth / 2 - Player.size / 2, .y = utils.screenHeight / 2 - Player.size / 2 },
            .target = rl.Vector2{ .x = x, .y = y },
            .rotation = 0,
            .zoom = 1,
        },
    };
}

pub fn follow(self: *Camera, player: *const Player) void {
    self.state.target.x += @divTrunc(player.x - self.state.target.x, speed);
    self.state.target.y += @divTrunc(player.y - self.state.target.y, speed);
}
