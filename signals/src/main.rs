mod game;
mod gfx;
mod processor;
mod tool;
mod ui;
mod world;

use game::Game;
use gfx::PosInfo;
use raylib::{
	ffi::{KeyboardKey, MouseButton},
	prelude::RaylibDraw,
};
use sui::{
	comp::fit::scrollable,
	core::{Event, Store},
	Layable,
};
use tool::Tool;

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

	let mut worlds_bar_cache = sui::core::Cached::default();
	let scroll_state = Store::new(Default::default());
	let mut game_retexture_counter = 0; // <- change this variable for the worlds_bar to regenerate

	let dbg_scroll_state = Store::new(Default::default());

	let mut inst_comp_cache = sui::core::Cached::default();
	let inst_scroll_state = Store::new(Default::default());
	let mut inst_comp_counter = 0; // <- change this variable for the instruction list to regenerate

	let mut delta = 0.0;

	let mut g_pos = PosInfo::default();

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

		if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_Q) {
			inst_comp_counter += 1;
			println!("triggering instruction recompute")
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

		let worlds_bar_h = 400 as f32 / 1980 as f32 * screen.ah as f32;
		let worlds_bar_h = worlds_bar_h as i32;
		// modified so width reflects the real width
		let mut worlds_bar_det = sui::Details {
			x: 0,
			y: screen.ah - worlds_bar_h - scrollable::SCROLLBAR_WIDTH as i32,
			aw: screen.aw,
			ah: worlds_bar_h + scrollable::SCROLLBAR_WIDTH as i32,
		};

		// don't be confused by the name, this code block mostly handles rendering
		let events = {
			let mut d = rl.begin_drawing(&thread);

			let page = ui::game_debug_ui(&game, dbg_scroll_state.clone());
			let dbg_ctx = sui::RootContext::new(
				&page,
				sui::Details {
					x: 0,
					y: 100,
					..Default::default()
				},
				1.0,
			);

			let worlds_bar = worlds_bar_cache.update_with_unchecked(
				(game_retexture_counter, worlds_bar_h),
				(&mut d, &game),
				|(_, height), (d, game)| ui::worlds_bar(d, game, height, scroll_state.clone()),
			);
			worlds_bar_det.aw = worlds_bar_det.aw.min(worlds_bar.size().0);
			let worlds_bar_ctx = sui::RootContext::new(worlds_bar, worlds_bar_det, 1.0);

			let inst_comp = inst_comp_cache.update_with_unchecked(
				inst_comp_counter,
				(&game, game.i, inst_scroll_state.clone()),
				|_, (game, world_id, inst_scroll_state)| {
					ui::inst_comp(game, world_id, inst_scroll_state)
				},
			);
			let inst_comp_w = inst_comp.size().0.max(100);
			let inst_comp_det = sui::Details {
				x: screen.aw - inst_comp_w,
				y: 100,
				aw: inst_comp_w,
				ah: inst_comp_w * 2,
			};
			let inst_comp_ctx = sui::RootContext::new(inst_comp, inst_comp_det, 1.0);

			// handled later, when there's no other references to game
			let events = dbg_ctx
				.handle_input_d(&mut d)
				.chain(worlds_bar_ctx.handle_input_d(&mut d))
				.chain(inst_comp_ctx.handle_input_d(&mut d));

			d.clear_background(gfx::BACKGROUND);

			gfx::render_world(&game.main, &mut d, pos_info);

			tool_select.render(&mut d, tool_select_det, Some(&tool));
			dbg_ctx.render(&mut d);
			worlds_bar_ctx.render(&mut d);
			inst_comp_ctx.render(&mut d);

			events.collect::<Vec<_>>()
		};

		{
			let (mouse_x, mouse_y) = (rl.get_mouse_x(), rl.get_mouse_y());

			let tool_select_trig = tool_select.tick(&mut rl, tool_select_det, &mut tool);
			if !tool_select_trig && !worlds_bar_det.is_inside(mouse_x, mouse_y) {
				let point_x = round(
					(mouse_x as f32 - pos_info.base.0 as f32)
						/ world::BLOCK_SIZE as f32
						/ pos_info.scale,
				);
				let point_y = round(
					(mouse_y as f32 - pos_info.base.1 as f32)
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

		for event_out in events {
			let n_to_id = |n: i32| match n {
				0 => Option::<usize>::None,
				n => Some(n as usize - 1),
			};

			println!("{} {event_out:?}", rl.get_time());
			match event_out {
				Event::Named {
					id: ui::worlds_bar::PLUS_CLICKED,
					..
				} => {
					let id = game.push();
					game.switch_main(id);
					game_retexture_counter += 1;
				}
				Event::Named {
					id: ui::worlds_bar::SWITCH_CLICKED,
					n,
				} => {
					let id = n_to_id(n);
					game.switch_main(id);
					game_retexture_counter += 1;
				}
				Event::Named {
					id: ui::worlds_bar::FOREIGN_CLICKED,
					n,
				} => {
					if let Some(id) = n_to_id(n) {
						tool = Tool::PlaceForeign(id)
					} else {
						println!("can't really place a foreign to main")
					}
				}
				_ => (),
			}
		}
	}
}
