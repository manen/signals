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
use sui::{comp::fit::scrollable, core::Store, Layable, LayableExt};
use tool::Tool;
use ui::{worlds_bar, SignalsEvent};

pub const TICK_TIME: f32 = 0.03;
pub const MOVE_UP: KeyboardKey = KeyboardKey::KEY_W;
pub const MOVE_DOWN: KeyboardKey = KeyboardKey::KEY_S;
pub const MOVE_LEFT: KeyboardKey = KeyboardKey::KEY_A;
pub const MOVE_RIGHT: KeyboardKey = KeyboardKey::KEY_D;
pub const TOOL_USE: MouseButton = MouseButton::MOUSE_BUTTON_LEFT;
pub const MOVE_AMOUNT: f32 = 5000.0;

const SAVE_PATH: &str = "./signals.snsv";

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

	let mut game = match game::saves::read_worlds(SAVE_PATH) {
		Ok(a) => Game::from_worlds(a).unwrap_or_else(|err| {
			eprintln!("failed to load game from worlds:\n{err}");
			Default::default()
		}),
		Err(err) => {
			eprintln!("failed to load worlds, using default\n{err}");
			Default::default()
		}
	};

	let mut tool: tool::Tool = Default::default();
	let tool_select = sui::SelectBar::new(tool::TOOLS);

	let mut worlds_bar = worlds_bar::WorldsBar::default();

	let mut dbg_cache = sui::core::Cached::default();
	let dbg_scroll_state = Store::new(Default::default());
	let mut inst_comp_counter = 0; // <- change this variable for the instruction list to regenerate

	let mut delta = 0.0;

	let mut g_pos = PosInfo::default();

	fn frame_dialog(comp: sui::Comp<'static>) -> sui::Comp<'static> {
		// bogi edition
		// .with_background(comp::Color::new(sui::color(242, 109, 133, 255)))
		// .margin(6)
		// .with_background(comp::Color::new(sui::color(242, 61, 93, 255)))

		use sui::comp;
		let comp = comp
			.margin(5)
			.with_background(comp::Color::new(sui::color(13, 13, 13, 255)))
			.margin(1)
			.with_background(comp::Color::new(sui::color(255, 255, 255, 255)))
			.clickable_fallback(|_| SignalsEvent::DialogFallback)
			.margin(2);

		sui::custom(comp)
	}

	let mut dialog_handler = sui::dialog::Handler::new(frame_dialog);
	let mut focus_handler = sui::form::focus_handler();

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
			match game.tick() {
				Ok(_) => (),
				Err(err) => {
					eprintln!("error while ticking game:\n{err}")
				}
			};
		}

		let (mouse_x, mouse_y) = (rl.get_mouse_x(), rl.get_mouse_y());
		let point_x = round(
			(mouse_x as f32 - pos_info.base.0 as f32) / world::BLOCK_SIZE as f32 / pos_info.scale,
		);
		let point_y = round(
			(mouse_y as f32 - pos_info.base.1 as f32) / world::BLOCK_SIZE as f32 / pos_info.scale,
		);

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

			let page = dbg_cache.update_with_unchecked(
				inst_comp_counter,
				(&game, dbg_scroll_state.clone()),
				|_, (game, dbg_scroll_state)| ui::game_debug_ui(game, dbg_scroll_state.clone()),
			);
			let dbg_ctx = sui::RootContext::new(
				&page,
				sui::Details {
					x: 0,
					y: 100,
					..Default::default()
				},
				1.0,
			);

			let worlds_bar_comp = worlds_bar.update(&mut d, &game, worlds_bar_h);
			worlds_bar_det.aw = worlds_bar_det.aw.min(worlds_bar_comp.size().0);
			let worlds_bar_ctx = worlds_bar.root_context(worlds_bar_det, 1.0);

			let dialog_ctx = dialog_handler.root_context();

			// handled later, when there's no other references to game
			// let events = dbg_ctx
			// 	.handle_input_d(&mut d)
			// 	.chain(worlds_bar_ctx.handle_input_d(&mut d))
			// 	.chain(dialog_ctx.handle_input_d(&mut d));
			let events = dialog_ctx
				.handle_input(&mut d, &focus_handler)
				.collect::<Vec<_>>();
			let events = if events.len() == 0 {
				worlds_bar_ctx
					.handle_input(&mut d, &focus_handler)
					.chain(dbg_ctx.handle_input(&mut d, &focus_handler))
					.collect()
			} else {
				events
			};

			d.clear_background(gfx::BACKGROUND);

			if let Some(main) = game.main() {
				gfx::render_world(&main, &mut d, pos_info, &game.drawmap);
			} else {
				// temporary text to differentiate a non-world from an empty world
				use sui::{comp, core::Layable};
				comp::Centered::new(comp::Text::new(
					if game.worlds().count() == 0 {
						"create a world using the + icon"
					} else {
						"select a world to start building"
					},
					32,
				))
				.render(&mut d, screen, 1.0);
			}

			tool_select.render(&mut d, tool_select_det, Some(&tool));
			dbg_ctx.render(&mut d);
			worlds_bar_ctx.render(&mut d);

			dialog_ctx.render(&mut d);

			sui::text(format!("({point_x}, {point_y})"), 32).render(
				&mut d,
				sui::Details {
					x: 0,
					y: 68,
					..Default::default()
				},
				1.0,
			);

			events
		};

		{
			let tool_select_trig = tool_select.tick(&mut rl, tool_select_det, &mut tool);
			if !tool_select_trig && !worlds_bar_det.is_inside(mouse_x, mouse_y) {
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
			let event_out = event_out.take();

			println!("{} {event_out:?}", rl.get_time());
			match event_out {
				Some(SignalsEvent::DialogCommand(command)) => {
					dialog_handler.run(command);
				}
				Some(SignalsEvent::FocusCommand(command)) => {
					command.apply(&mut focus_handler);
				}
				Some(SignalsEvent::NewWorld) => {
					let wid = game.push();
					game.switch_main(wid);
					worlds_bar.clear_cache();
				}
				Some(SignalsEvent::SwitchToWorld(wid)) => {
					game.switch_main(wid);
					worlds_bar.clear_cache();
				}
				Some(SignalsEvent::PlaceWorld(wid)) => tool = Tool::PlaceForeign(wid),
				Some(SignalsEvent::WorldsBarFallback) | Some(SignalsEvent::DialogFallback) => {}
				None => eprintln!("ui returned invalid returnevent"),
			}
		}
	}
	let save = game::saves::write_worlds(&game.worlds).expect("couldn't serialize progress");
	std::fs::write(SAVE_PATH, &save).expect("couldn't save progress to file");
}
