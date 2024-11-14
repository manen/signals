mod gfx;
mod tool;
mod world;

use gfx::PosInfo;
use raylib::prelude::RaylibDraw;

fn main() {
	let (mut rl, thread) = raylib::init()
		.size(640, 480)
		.title("signals")
		.resizable()
		.build();
	rl.set_window_position((1920.0 * 1.3) as i32, (1920.0 * 0.6) as i32);

	let mut world = world::World::default();

	let mut tool: tool::Tool = Default::default();
	let tool_select = sui::SelectBar::new(tool::TOOLS);

	// world.mut_at(-2, 2);

	let mut delta = 0.0;
	let mut moves = Vec::new();

	let mut g_pos = PosInfo::default();

	let mut page = sui::layout::Page::empty();
	page.push::<sui::Text>(("szia", 14));
	page.push::<sui::Text>(("SZIA", 34));
	page.push::<sui::Text>(("SZIA", 12));

	while !rl.window_should_close() {
		let screen = sui::Details::window(rl.get_render_width(), unsafe {
			raylib::ffi::GetRenderHeight()
		});
		let tool_select_det = screen.from_top(30);

		let screen_middle = (screen.aw / 2, screen.ah / 2);
		let pos_info = g_pos.add(screen_middle.0, screen_middle.1);

		let round = |a: f32| {
			if a < 0.0 {
				a as i32 - 1
			} else {
				a as i32
			}
		};

		{
			if !tool_select.tick(&mut rl, tool_select_det, &mut tool) {
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
			moves = world
				.tick(moves)
				.into_iter()
				.map(|mov| match mov {
					world::Move::Output { id, signal } => {
						// so sometimes it works sometimes it gets stuck in an infinite loop yk whatever it feels like
						world::Move::Input { id, signal }
					}
					mov => mov,
				})
				.collect();
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(consts::BACKGROUND);

		gfx::render_world(&world, &mut d, pos_info);

		tool_select.render(&mut d, tool_select_det, Some(&tool));
		page.render(&mut d, 100, 200, 1);
	}
}
