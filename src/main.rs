mod consts;
mod gfx;
mod tool;
mod world;

use raylib::{color::Color, prelude::RaylibDraw};

fn main() {
	let (mut rl, thread) = raylib::init()
		.size(640, 480)
		.title("signals")
		.resizable()
		.build();
	rl.set_window_position((1920.0 * 1.3) as i32, (1920.0 * 0.6) as i32);

	let mut world = world::World::default();
	let mut tool: Option<tool::Tool> = None;

	let world_offset = (50, 100);

	let mut delta = 0.0;
	let mut moves = Vec::new();

	while !rl.window_should_close() {
		if rl.is_key_pressed(consts::TOOL_SWITCH) {
			tool = match tool {
				Some(tool) => Some(tool.rotate()),
				_ => Some(tool::Tool::default()),
			};
		}
		if let Some(tool) = &mut tool {
			let x = (rl.get_mouse_x().max(0) - world_offset.0) / world::BLOCK_SIZE;
			let y = (rl.get_mouse_y().max(0) - world_offset.1) / world::BLOCK_SIZE;
			if rl.is_mouse_button_down(consts::TOOL_USE) {
				tool.down(x, y, &mut world);
			}
			if rl.is_mouse_button_pressed(consts::TOOL_USE) {
				tool.pressed(x, y, &mut world);
			}
			if rl.is_mouse_button_released(consts::TOOL_USE) {
				tool.released(x, y, &mut world);
			}
		} else {
		}

		delta += rl.get_frame_time();
		for _ in 0..(delta / consts::TICK_TIME) as i32 {
			delta -= consts::TICK_TIME;
			moves = world.tick(moves);
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(consts::BACKGROUND);

		gfx::render_world(&world, &mut d, world_offset.0, world_offset.1);

		d.draw_text(&format!("{tool:?}"), 0, 0, 20, Color::WHITE);
		d.draw_text(&format!("{moves:?}"), 0, 20, 20, Color::WHITE);
	}
}
