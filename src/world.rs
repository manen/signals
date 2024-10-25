use raylib::{color::Color, drawing::RaylibDrawHandle, prelude::RaylibDraw};

use crate::consts;

pub const CHUNK_SIZE: usize = 16;
pub const BLOCK_SIZE: i32 = 32;

#[derive(Copy, Clone, Debug, Default)]
pub struct Chunk([[Block; CHUNK_SIZE]; CHUNK_SIZE]);
impl Chunk {
	pub fn checkerboard() -> Self {
		let mut chunk = Self::default();

		let mut s = false;
		let mut dir = Direction::Right;
		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				dir = dir.rotate();
				s = !s;
				chunk.0[px][py] = Block::Wire(dir, s);
			}
			dir = dir.rotate();
			s = !s;
		}

		chunk
	}

	pub fn tick(&mut self) {
		let old_self = *self;

		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				let a = old_self.at(px, py).unwrap();

				if a.activated() {
					let (ox, oy) = a.target_offset().unwrap();
					self.mut_at(
						(px as i32 + ox).max(0).min(15) as usize,
						(py as i32 + oy).max(0).min(15) as usize,
					)
					.map(|x| x.power());
				}
			}
		}
	}

	pub fn at(&self, x: usize, y: usize) -> Option<&Block> {
		if x > 15 || y > 15 {
			return None;
		}
		Some(&self.0[x][y])
	}
	pub fn mut_at(&mut self, x: usize, y: usize) -> Option<&mut Block> {
		if x > 15 || y > 15 {
			return None;
		}
		Some(&mut self.0[x][y])
	}
	pub fn map_at(&mut self, x: usize, y: usize, f: impl FnOnce(Block) -> Block) {
		if x > 15 || y > 15 {
			return;
		}
		self.0[x][y] = f(self.0[x][y]);
	}
	pub fn draw_at(&self, d: &mut RaylibDrawHandle, x: i32, y: i32) {
		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				self.0[px][py].draw_at(d, x + px as i32 * BLOCK_SIZE, y + py as i32 * BLOCK_SIZE);
			}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Block {
	#[default]
	Nothing,
	Wire(Direction, bool),
}
impl Block {
	pub fn activated(&self) -> bool {
		match self {
			Block::Wire(_, a) => *a,
			_ => false,
		}
	}
	pub fn target_offset(&self) -> Option<(i32, i32)> {
		match self {
			Self::Wire(dir, _) => Some(match dir {
				Direction::Right => (1, 0),
				Direction::Bottom => (0, 1),
				Direction::Left => (-1, 0),
				Direction::Top => (0, -1),
			}),
			_ => None,
		}
	}
	pub fn power(&mut self) {
		match self {
			Self::Wire(_, s) => *s = true,
			_ => {}
		};
	}

	pub fn draw_at(&self, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
		match self {
			Block::Nothing => {}
			Block::Wire(dir, state) => {
				let horizontal = match dir {
					Direction::Bottom | Direction::Top => false,
					_ => true,
				};
				let off = BLOCK_SIZE / 4;
				let x_off = if !horizontal { off } else { 0 };
				let y_off = if horizontal { off } else { 0 };

				let color = if *state {
					consts::WIRE_ON
				} else {
					consts::WIRE_OFF
				};

				d.draw_rectangle(
					base_x + x_off,
					base_y + y_off,
					BLOCK_SIZE - x_off * 2,
					BLOCK_SIZE - y_off * 2,
					color,
				);

				let c = match dir {
					Direction::Right => "r",
					Direction::Bottom => "b",
					Direction::Left => "l",
					Direction::Top => "t",
				};
				d.draw_text(c, base_x + x_off, base_y + y_off, 8, Color::WHITE);
			}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
	Right,
	Bottom,
	Left,
	Top,
}
impl Direction {
	/// clockwise
	pub fn rotate(self) -> Self {
		match self {
			Direction::Right => Direction::Bottom,
			Direction::Bottom => Direction::Left,
			Direction::Left => Direction::Top,
			Direction::Top => Direction::Right,
		}
	}
}
