use raylib::{RaylibHandle, RaylibThread};
use sui::Layable;

pub mod ctx;
pub use ctx::Context;

/// main function of the library, creates a raylib context
/// and creates a [Context]
///
/// to start execution, call [Context::start]
pub fn ctx<L: Layable>(layable: L) -> Context<L> {
	let (rl, thread) = rl();
	Context::new(layable, rl, thread)
}

pub fn rl() -> (RaylibHandle, RaylibThread) {
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

	(rl, thread)
}
