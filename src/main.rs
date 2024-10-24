mod colors;
mod world;

use raylib::{color::Color, prelude::RaylibDraw};

fn main() {
	let (mut rl, thread) = raylib::init().size(640, 480).title("signals").build();
	rl.set_window_position((1920.0 * 1.3) as i32, (1920.0 * 0.6) as i32);

	let chunk = world::Chunk::checkerboard();

	while !rl.window_should_close() {
		let mut d = rl.begin_drawing(&thread);

		d.clear_background(Color::BLACK);
		chunk.draw_at(&mut d, 10, 10);
	}
}
