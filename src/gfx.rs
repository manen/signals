use raylib::prelude::{RaylibDraw, RaylibDrawHandle};

use crate::{
	consts,
	world::{self},
};

pub fn render_world(world: &world::World, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
	for (coords, chunk) in world.chunks() {
		render_chunk(
			&chunk,
			d,
			base_x + coords.0 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
			base_y + coords.1 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
		);
	}
}

pub fn render_chunk(chunk: &world::Chunk, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
	for px in 0..world::CHUNK_SIZE as i32 {
		for py in 0..world::CHUNK_SIZE as i32 {
			let (base_x, base_y) = (
				base_x + px * world::BLOCK_SIZE,
				base_y + py * world::BLOCK_SIZE,
			);
			chunk.at(px, py).map(|b| render_block(b, d, base_x, base_y));

			if consts::DEBUG_CHUNKS {
				use raylib::prelude::RaylibDraw;
				d.draw_text(
					&format!("{px} {py}"),
					base_x,
					base_y,
					6,
					raylib::color::Color::WHITE,
				);
			}
		}
	}
}

pub fn render_block(block: &world::Block, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
	match block {
		world::Block::Nothing => {}
		world::Block::Wire(dir, ticks) => {
			let horizontal = match dir {
				world::Direction::Bottom | world::Direction::Top => false,
				_ => true,
			};
			let off = world::BLOCK_SIZE / 4;
			let x_off = if !horizontal { off } else { 0 };
			let y_off = if horizontal { off } else { 0 };

			let color = if *ticks < 3 {
				consts::WIRE_ON
			} else {
				consts::WIRE_OFF
			};

			d.draw_rectangle(
				base_x + x_off,
				base_y + y_off,
				world::BLOCK_SIZE - x_off * 2,
				world::BLOCK_SIZE - y_off * 2,
				color,
			);

			if consts::DEBUG_WIRES {
				use raylib::color::Color;
				let c_dir = match dir {
					world::Direction::Right => "r",
					world::Direction::Bottom => "b",
					world::Direction::Left => "l",
					world::Direction::Top => "t",
				};
				let c = format!("{c_dir} {ticks}");
				d.draw_text(&c, base_x + x_off, base_y + y_off, 8, Color::WHITE);
			}
		}
		world::Block::Switch(state) => {
			d.draw_rectangle(
				base_x,
				base_y,
				world::BLOCK_SIZE,
				world::BLOCK_SIZE,
				if *state {
					consts::SWITCH_ON
				} else {
					consts::SWITCH_OFF
				},
			);
		}
		world::Block::Not(state) => {
			d.draw_rectangle(
				base_x,
				base_y,
				world::BLOCK_SIZE,
				world::BLOCK_SIZE,
				consts::NOT_BASE,
			);

			let excl_color = if *state {
				consts::NOT_ON
			} else {
				consts::NOT_OFF
			};
			let excl_width = 6;
			let excl_height = 24;
			let excl_point = 4;

			let excl_start_x = base_x + world::BLOCK_SIZE / 2 - excl_width / 2;
			let excl_start_y = base_y + (world::BLOCK_SIZE - excl_height) / 2;

			d.draw_rectangle(
				excl_start_x,
				excl_start_y,
				excl_width,
				excl_height - excl_point * 2,
				excl_color,
			);
			d.draw_rectangle(
				excl_start_x,
				excl_start_y + excl_height - excl_point,
				excl_width,
				excl_point,
				excl_color,
			);

			if consts::DEBUG_NOT {
				d.draw_text(&format!("{block:?}"), base_x, base_y, 6, excl_color);
			}
		}
	}
}
