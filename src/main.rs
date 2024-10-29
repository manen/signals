mod consts;
mod gfx;
mod tool;
mod world;

use gfx::PosInfo;
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

	// world.mut_at(-2, 2);

	let mut delta = 0.0;
	let mut moves = Vec::new();

	let mut g_pos = PosInfo::default();

	while !rl.window_should_close() {
		let screen_middle = (
			rl.get_render_width() / 2,
			unsafe { raylib::ffi::GetRenderHeight() } / 2,
		);
		let pos_info = g_pos.add(screen_middle.0, screen_middle.1);

		if rl.is_key_pressed(consts::TOOL_SWITCH) {
			tool = match tool {
				Some(tool) => Some(tool.rotate()),
				_ => Some(tool::Tool::default()),
			};
		}

		let round = |a: f32| {
			if a < 0.0 {
				a as i32 - 1
			} else {
				a as i32
			}
		};

		let point_x = round(
			(rl.get_mouse_x() as f32 - pos_info.base.0 as f32)
				/ world::BLOCK_SIZE as f32
				/ pos_info.scale,
		);
		let point_y = round(
			(rl.get_mouse_y() as f32 - pos_info.base.1 as f32)
				/ world::BLOCK_SIZE as f32
				/ pos_info.scale,
		);

		// let point_x = if point_x < 0 { point_x } else { point_x };
		// let point_y = if point_y < 0 { point_y } else { point_y };

		// whole lotta bugs in negativeland

		if let Some(tool) = &mut tool {
			if rl.is_mouse_button_down(consts::TOOL_USE) {
				tool.down(point_x, point_y, &mut world);
			}
			if rl.is_mouse_button_pressed(consts::TOOL_USE) {
				tool.pressed(point_x, point_y, &mut world);
			}
			if rl.is_mouse_button_released(consts::TOOL_USE) {
				tool.released(point_x, point_y, &mut world);
			}
		}

		g_pos.scale *= 1.0 + (rl.get_mouse_wheel_move() * 0.1);

		let move_amount = (consts::MOVE_AMOUNT * rl.get_frame_time()) as i32;
		if rl.is_key_down(consts::MOVE_UP) {
			g_pos.base.1 += move_amount;
		}
		if rl.is_key_down(consts::MOVE_DOWN) {
			g_pos.base.1 -= move_amount;
		}
		if rl.is_key_down(consts::MOVE_LEFT) {
			g_pos.base.0 += move_amount;
		}
		if rl.is_key_down(consts::MOVE_RIGHT) {
			g_pos.base.0 -= move_amount;
		}

		delta += rl.get_frame_time();
		for _ in 0..(delta / consts::TICK_TIME) as i32 {
			delta -= consts::TICK_TIME;
			moves = world.tick(moves);
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(consts::BACKGROUND);

		gfx::render_world(&world, &mut d, pos_info);

		d.draw_text(&format!("{tool:?}"), 0, 0, 20, Color::WHITE);
		d.draw_text(
			&format!(
				"{point_x} {point_y} @ [{}, {}]",
				// world::world_coords_into_chunk_coords(point_x, point_y),
				d.get_mouse_x(),
				d.get_mouse_y()
			),
			0,
			20,
			20,
			Color::WHITE,
		);
	}
}
