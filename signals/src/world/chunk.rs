use crate::world::*;

use raylib::drawing::RaylibDrawHandle;

pub const CHUNK_SIZE: usize = 16;
pub const BLOCK_SIZE: i32 = 32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Chunk<T = Block>([[T; CHUNK_SIZE]; CHUNK_SIZE]);
impl<T> Chunk<T> {
	pub const fn new(array: [[T; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
		Self(array)
	}

	pub fn at(&self, x: i32, y: i32) -> Option<&T> {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return None;
		}
		Some(&self.0[x as usize][y as usize])
	}
	pub fn mut_at(&mut self, x: i32, y: i32) -> Option<&mut T> {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return None;
		}
		Some(&mut self.0[x as usize][y as usize])
	}
	pub fn map_at_b(&mut self, x: i32, y: i32, f: impl FnOnce(&T) -> T) {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return;
		}
		let (x, y) = (x as usize, y as usize);

		self.0[x][y] = f(&self.0[x][y]);
	}
}
impl<T: Copy> Chunk<T> {
	pub fn map_at(&mut self, x: i32, y: i32, f: impl FnOnce(T) -> T) {
		if x > 15 || y > 15 || x < 0 || y < 0 {
			return;
		}
		let (x, y) = (x as usize, y as usize);

		self.0[x][y] = f(self.0[x][y])
	}
}
impl Chunk<Block> {
	#[allow(dead_code)]
	pub fn checkerboard() -> Self {
		let mut chunk = Self::default();

		let mut s = false;
		let mut dir = Direction::Right;
		for px in 0..CHUNK_SIZE {
			for py in 0..CHUNK_SIZE {
				dir = dir.rotate();
				s = !s;
				chunk.0[px][py] = Block::Wire(dir);
			}
			dir = dir.rotate();
			s = !s;
		}

		chunk
	}
	pub fn tick(&mut self, mut push_move: impl FnMut(i32, i32, Option<Direction>, Signal)) {
		let old_self = *self;
		for x in 0..CHUNK_SIZE as i32 {
			for y in 0..CHUNK_SIZE as i32 {
				let a = crate::continue_on_none!(old_self.at(x, y));

				if let Some(b) = a.tick(|lx, ly, signal| {
					push_move(
						x + lx,
						y + ly,
						Direction::from_rel((lx, ly)).map(|dir| dir.reverse()),
						signal,
					)
				}) {
					*crate::continue_on_none!(self.mut_at(x, y)) = b;
				}
			}
		}
	}
}
