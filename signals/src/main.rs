mod gfx;
mod tool;
mod world;

use gfx::PosInfo;
use raylib::{
	ffi::{KeyboardKey, MouseButton},
	prelude::RaylibDraw,
};

pub const TICK_TIME: f32 = 0.03;
pub const MOVE_UP: KeyboardKey = KeyboardKey::KEY_W;
pub const MOVE_DOWN: KeyboardKey = KeyboardKey::KEY_S;
pub const MOVE_LEFT: KeyboardKey = KeyboardKey::KEY_A;
pub const MOVE_RIGHT: KeyboardKey = KeyboardKey::KEY_D;
pub const TOOL_USE: MouseButton = MouseButton::MOUSE_BUTTON_LEFT;
pub const MOVE_AMOUNT: f32 = 5000.0;

fn main() {
	let (start_width, start_height) = (640, 480);

	let (mut rl, thread) = raylib::init()
		.size(start_width, start_height)
		.title("signals")
		.resizable()
		.build();

	{
		// center window on screen
		let monitor = unsafe { raylib::ffi::GetCurrentMonitor() };
		let raylib::ffi::Vector2 { x: m_x, y: m_y } =
			unsafe { raylib::ffi::GetMonitorPosition(monitor) };
		let m_width = unsafe { raylib::ffi::GetMonitorWidth(monitor) };
		let m_height = unsafe { raylib::ffi::GetMonitorHeight(monitor) };

		rl.set_window_position(
			m_x as i32 + m_width / 2 - start_width / 2,
			m_y as i32 + m_height / 2 - start_height / 2,
		);
	}

	let mut world = world::RenderedWorld::default();

	let mut tool: tool::Tool = Default::default();
	let tool_select = sui::SelectBar::new(tool::TOOLS);

	// world.mut_at(-2, 2);

	let mut delta = 0.0;
	let mut moves = Vec::new();

	let mut g_pos = PosInfo::default();

	// let mut page = sui::layout::Page::empty();
	// page.push::<sui::Text>(("szia", 14));
	// page.push::<sui::Text>(("SZIA", 34));
	// page.push::<sui::Text>(("SZIA", 12));

	let page = sui::page(vec![
		sui::text("szia", 14),
		sui::text("SZIA", 34),
		sui::comp::Compatible::into_comp(sui::comp::Clickable::new(
			sui::text("SZIA", 12),
			"clicked",
			0,
		)),
		sui::text("SZIA", 45),
	]);

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

				if rl.is_mouse_button_down(TOOL_USE) {
					tool.down(point_x, point_y, world.as_mut());
				}
				if rl.is_mouse_button_pressed(TOOL_USE) {
					tool.pressed(point_x, point_y, world.as_mut());
				}
				if rl.is_mouse_button_released(TOOL_USE) {
					tool.released(point_x, point_y, world.as_mut());
				}
			}
		}

		g_pos.scale *= 1.0 + (rl.get_mouse_wheel_move() * 0.1);

		let move_amount = (MOVE_AMOUNT * rl.get_frame_time()) as i32;
		if rl.is_key_down(MOVE_UP) {
			g_pos.base.1 += move_amount;
		}
		if rl.is_key_down(MOVE_DOWN) {
			g_pos.base.1 -= move_amount;
		}
		if rl.is_key_down(MOVE_LEFT) {
			g_pos.base.0 += move_amount;
		}
		if rl.is_key_down(MOVE_RIGHT) {
			g_pos.base.0 -= move_amount;
		}

		delta += rl.get_frame_time();
		for _ in 0..(delta / TICK_TIME) as i32 {
			delta -= TICK_TIME;
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

		let scale = (rl.get_time() % 2.0) as f32;
		let event_out = sui::handle_input(&page, &mut rl, 0, 100, scale);
		if let Some(event_out) = event_out {
			println!("{} {event_out:?}", rl.get_time());
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(gfx::BACKGROUND);

		gfx::render_world(&world, &mut d, pos_info);

		tool_select.render(&mut d, tool_select_det, Some(&tool));
		sui::render_root(&page, &mut d, 0, 100, scale);
	}
}
