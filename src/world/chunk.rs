use crate::world::*;

use raylib::drawing::RaylibDrawHandle;

pub const CHUNK_SIZE: usize = 16;
pub const BLOCK_SIZE: i32 = 32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Chunk([[Block; CHUNK_SIZE]; CHUNK_SIZE]);
impl Chunk {
	#[allow(dead_code)]
	pub fn checkerboard() -> Self {
		let mut chunk = Self::default();

		let mut s = false;
		let mut dir = Direction::Right;
		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				dir = dir.rotate();
				s = !s;
				chunk.0[px][py] = Block::Wire(dir, s as u8);
			}
			dir = dir.rotate();
			s = !s;
		}

		chunk
	}
	pub fn tick(&mut self, mut push_move: impl FnMut(i32, i32, Signal)) {
		let old_self = *self;
		for x in 0..CHUNK_SIZE as i32 {
			for y in 0..CHUNK_SIZE as i32 {
				let a = crate::continue_on_none!(old_self.at(x, y));

				if let Some(b) = a.tick(|lx, ly, signal| push_move(x + lx, y + ly, signal)) {
					*crate::continue_on_none!(self.mut_at(x, y)) = b;
				}
			}
		}
	}

	pub fn at(&self, x: i32, y: i32) -> Option<&Block> {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return None;
		}
		Some(&self.0[x as usize][y as usize])
	}
	pub fn mut_at(&mut self, x: i32, y: i32) -> Option<&mut Block> {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return None;
		}
		Some(&mut self.0[x as usize][y as usize])
	}
	pub fn map_at(&mut self, x: i32, y: i32, f: impl FnOnce(Block) -> Block) {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return;
		}
		let (x, y) = (x as usize, y as usize);

		self.0[x][y] = f(self.0[x][y]);
	}
	pub fn draw_at(&self, d: &mut RaylibDrawHandle, x: i32, y: i32) {
		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				let (base_x, base_y) = (x + px as i32 * BLOCK_SIZE, y + py as i32 * BLOCK_SIZE);
				self.0[px][py].draw_at(d, base_x, base_y);

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
