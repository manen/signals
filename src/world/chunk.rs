use crate::world::*;

use raylib::drawing::RaylibDrawHandle;

pub const CHUNK_SIZE: usize = 16;
pub const BLOCK_SIZE: i32 = 32;

#[derive(Copy, Clone, Debug, Default)]
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

	pub fn tick(&mut self, moves: Moves) -> Moves {
		let mut new_moves = Moves::new();
		macro_rules! gen_push_move {
			($px:expr, $py:expr) => {
				|x, y, signal| {
					let mov = Move::new(
						(($px as i32 + x), ($py as i32 + y)),
						Direction::from_rel((x, y)).map(|dir| dir.reverse()),
						signal,
					);
					if !new_moves.contains(&mov) {
						new_moves.push(mov)
					}
				}
			};
		}

		macro_rules! continue_on_none {
			($expr:expr) => {
				match $expr {
					None => continue,
					Some(a) => a,
				}
			};
		}

		for mov in moves {
			let (x, y) = mov.to;
			let a = continue_on_none!(self.at(x, y));

			*continue_on_none!(self.mut_at(x, y)) =
				if let Some(b) = a.pass(mov.signal, mov.from, gen_push_move!(x, y)) {
					b
				} else {
					*a
				};
		}
		let old_self = *self;
		for px in 0..CHUNK_SIZE as i32 {
			for py in 0..CHUNK_SIZE as i32 {
				let a = continue_on_none!(old_self.at(px, py));

				*continue_on_none!(self.mut_at(px, py)) =
					if let Some(b) = a.tick(gen_push_move!(px, py)) {
						b
					} else {
						*a
					};
			}
		}

		new_moves
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

				// use raylib::prelude::RaylibDraw;
				// d.draw_text(
				// 	&format!("{px} {py}"),
				// 	base_x,
				// 	base_y,
				// 	6,
				// 	raylib::color::Color::WHITE,
				// );
			}
		}
	}
}
