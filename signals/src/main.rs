mod game;
mod gfx;
mod tool;
mod ui;
mod world;

use game::{Game, IngameWorld};
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

	let mut game = game::Game::default();

	let mut tool: tool::Tool = Default::default();
	let tool_select = sui::SelectBar::new(tool::TOOLS);
	let foreign_select = sui::SelectBar::new(tool::FOREIGNS);

	// world.mut_at(-2, 2);

	let mut delta = 0.0;
	// let mut moves = Vec::new();

	let mut g_pos = PosInfo::default();

	while !rl.window_should_close() {
		if let Some(ch) = rl.get_char_pressed() {
			if let Some(num) = ch.to_digit(10) {
				game.switch_main(if num == 0 { None } else { Some(num as usize) });
				let i = game.i;
				game.moves = IngameWorld::generate(&mut game, i);
			}
		}

		let screen = sui::Details::window(rl.get_render_width(), unsafe {
			raylib::ffi::GetRenderHeight()
		});
		let mut select_det = screen.from_top(60).split_h(2);
		let (tool_select_det, foreign_select_det) =
			(select_det.next().unwrap(), select_det.next().unwrap());

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
			let foreign_select_trig = foreign_select.tick(&mut rl, foreign_select_det, &mut tool);
			let tool_select_trig = tool_select.tick(&mut rl, tool_select_det, &mut tool);

			if !tool_select_trig && !foreign_select_trig {
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
					tool.down(point_x, point_y, &mut game);
				}
				if rl.is_mouse_button_pressed(TOOL_USE) {
					tool.pressed(point_x, point_y, &mut game);
				}
				if rl.is_mouse_button_released(TOOL_USE) {
					tool.released(point_x, point_y, &mut game);
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
			game.tick();
		}

		let page = ui::game_debug_ui(&game);

		let scale = 1.0;
		let event_out = sui::handle_input(&page, &mut rl, 0, 100, scale);
		if let Some(event_out) = event_out {
			println!("{} {event_out:?}", rl.get_time());
		}

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(gfx::BACKGROUND);

		gfx::render_world(&game.main, &mut d, pos_info);

		tool_select.render(&mut d, tool_select_det, Some(&tool));
		foreign_select.render(&mut d, foreign_select_det, Some(&tool));
		d.draw_rectangle_lines(
			foreign_select_det.x,
			foreign_select_det.y,
			foreign_select_det.aw,
			foreign_select_det.ah,
			gfx::NOT_BASE,
		);

		sui::render_root(&page, &mut d, 0, 100, scale);
		std::mem::drop(page);

		let worlds_bar_h = 400 as f32 / 1980 as f32 * screen.ah as f32;
		let worlds_bar_h = worlds_bar_h as _;
		sui::render_root(
			&ui::worlds_bar(&game, worlds_bar_h),
			&mut d,
			0,
			screen.ah - worlds_bar_h,
			scale,
		);
	}
}
