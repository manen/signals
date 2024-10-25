mod consts;
mod edit;
mod world;

use raylib::{color::Color, prelude::RaylibDraw};

fn main() {
	let (mut rl, thread) = raylib::init().size(640, 480).title("signals").build();
	rl.set_window_position((1920.0 * 1.3) as i32, (1920.0 * 0.6) as i32);

	let mut chunk = world::Chunk::default();
	let mut tool = Option::<edit::Tool>::None;

	let world_offset = (0, 20);

	let mut delta = 0.0;

	while !rl.window_should_close() {
		if rl.is_key_pressed(consts::TOOL_SWITCH) {
			tool = match tool {
				Some(tool) => Some(tool.rotate()),
				_ => Some(edit::Tool::default()),
			};
		}
		if let Some(tool) = &mut tool {
			let x = ((rl.get_mouse_x().max(0) - world_offset.0) / world::BLOCK_SIZE) as usize;
			let y = ((rl.get_mouse_y().max(0) - world_offset.1) / world::BLOCK_SIZE) as usize;
			if rl.is_mouse_button_down(consts::TOOL_USE) {
				tool.down(x, y, &mut chunk);
			}
			if rl.is_mouse_button_pressed(consts::TOOL_USE) {
				tool.pressed(x, y, &mut chunk);
			}
			if rl.is_mouse_button_released(consts::TOOL_USE) {
				tool.released(x, y, &mut chunk);
			}
		}

		delta += rl.get_frame_time();
		if delta > 0.2 {
			delta -= 0.2;
			chunk.tick();
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(consts::BACKGROUND);

		chunk.draw_at(&mut d, world_offset.0, world_offset.1);

		d.draw_text(&format!("{tool:?}"), 0, 0, 20, Color::WHITE);
	}
}
