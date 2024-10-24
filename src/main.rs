mod consts;
mod edit;
mod world;

use raylib::{color::Color, prelude::RaylibDraw};

fn main() {
	let (mut rl, thread) = raylib::init().size(640, 480).title("signals").build();
	rl.set_window_position((1920.0 * 1.3) as i32, (1920.0 * 0.6) as i32);

	let mut chunk = world::Chunk::checkerboard();
	let mut tool = None;

	while !rl.window_should_close() {
		if rl.is_key_pressed(consts::TOOL_SWITCH) {
			tool = match tool {
				None => Some(edit::Tool::Place(world::Block::Nothing)),
				Some(tool) => Some(tool.rotate()),
			};
		}
		if let Some(tool) = tool {
			if rl.is_mouse_button_down(consts::TOOL_USE) {
				tool.down(
					(rl.get_mouse_x() / world::BLOCK_SIZE) as _,
					(rl.get_mouse_y() / world::BLOCK_SIZE) as _,
					&mut chunk,
				);
			}
			if rl.is_mouse_button_pressed(consts::TOOL_USE) {
				tool.pressed(
					(rl.get_mouse_x() / world::BLOCK_SIZE) as _,
					(rl.get_mouse_y() / world::BLOCK_SIZE) as _,
					&mut chunk,
				);
			}
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(consts::BACKGROUND);

		chunk.draw_at(&mut d, 0, 0);

		d.draw_text(&format!("{tool:?}"), 0, 0, 20, Color::WHITE);
	}
}
