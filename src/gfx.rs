use raylib::prelude::{RaylibDraw, RaylibDrawHandle};

use crate::{
	consts,
	world::{self},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PosInfo {
	pub base: (i32, i32),
	pub scale: f32,
}
impl Default for PosInfo {
	fn default() -> Self {
		PosInfo {
			base: (0, 0),
			scale: 1.0,
		}
	}
}
impl PosInfo {
	pub fn transform(&self, x: i32, y: i32) -> Self {
		self.add(self.scale(x), self.scale(y))
	}
	pub fn add(&self, x: i32, y: i32) -> Self {
		PosInfo {
			base: (self.base.0 + x, self.base.1 + y),
			scale: self.scale,
		}
	}
	pub fn scale(&self, n: i32) -> i32 {
		(self.scale * n as f32) as i32
	}
}

pub fn render_world(world: &world::World, d: &mut RaylibDrawHandle, pos_info: PosInfo) {
	for (coords, chunk) in world.chunks() {
		render_chunk(
			&chunk,
			d,
			pos_info.transform(
				coords.0 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
				coords.1 * world::CHUNK_SIZE as i32 * world::BLOCK_SIZE as i32,
			),
		);
	}
}

pub fn render_chunk(chunk: &world::Chunk, d: &mut RaylibDrawHandle, pos_info: PosInfo) {
	for px in 0..world::CHUNK_SIZE as i32 {
		for py in 0..world::CHUNK_SIZE as i32 {
			let pos_info = pos_info.transform(px * world::BLOCK_SIZE, py * world::BLOCK_SIZE);
			let (base_x, base_y) = pos_info.base;
			chunk.at(px, py).map(|b| render_block(b, d, pos_info));

			if consts::DEBUG_CHUNKS {
				use raylib::prelude::RaylibDraw;
				d.draw_text(
					&format!("{px} {py}"),
					base_x,
					base_y,
					(6.0 * pos_info.scale) as i32,
					raylib::color::Color::WHITE,
				);
			}
		}
	}
}

pub fn render_block(block: &world::Block, d: &mut RaylibDrawHandle, pos_info: PosInfo) {
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

			let pos_info = pos_info.transform(x_off, y_off);
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE - x_off * 2),
				pos_info.scale(world::BLOCK_SIZE - y_off * 2),
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
				d.draw_text(
					&c,
					pos_info.base.0,
					pos_info.base.1,
					pos_info.scale(8),
					Color::WHITE,
				);
			}
		}
		world::Block::Switch(state) => {
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				if *state {
					consts::SWITCH_ON
				} else {
					consts::SWITCH_OFF
				},
			);
		}
		world::Block::Not(state) => {
			d.draw_rectangle(
				pos_info.base.0,
				pos_info.base.1,
				pos_info.scale(world::BLOCK_SIZE),
				pos_info.scale(world::BLOCK_SIZE),
				consts::NOT_BASE,
			);

			let excl_color = if *state {
				consts::NOT_ON
			} else {
				consts::NOT_OFF
			};
			let excl_width = pos_info.scale(6);
			let excl_height = pos_info.scale(24);
			let excl_point = pos_info.scale(4);

			let excl_start_x = pos_info.base.0 + world::BLOCK_SIZE / 2 - excl_width / 2;
			let excl_start_y = pos_info.base.1 + (world::BLOCK_SIZE - excl_height) / 2;

			d.draw_rectangle(
				excl_start_x,
				excl_start_y,
				pos_info.scale(excl_width),
				pos_info.scale(excl_height - excl_point * 2),
				excl_color,
			);
			d.draw_rectangle(
				excl_start_x,
				excl_start_y + excl_height - excl_point,
				pos_info.scale(excl_width),
				pos_info.scale(excl_point),
				excl_color,
			);

			if consts::DEBUG_NOT {
				d.draw_text(
					&format!("{block:?}"),
					pos_info.base.0,
					pos_info.base.0,
					pos_info.scale(6),
					excl_color,
				);
			}
		}
	}
}
