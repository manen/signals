use raylib::prelude::RaylibDrawHandle;

use crate::world::*;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Signal;
#[derive(Debug, PartialEq, Eq)]
pub struct Move {
	pub to: (i32, i32),
	pub from: Option<Direction>,
	pub signal: Signal,
}
impl Move {
	pub fn new(to: (i32, i32), from: Option<Direction>, signal: Signal) -> Self {
		Self { to, from, signal }
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct World {
	chunks: HashMap<(i32, i32), Chunk>,
}
impl World {
	pub fn tick(&mut self, moves: Vec<Move>) -> Vec<Move> {
		let mut new_moves = Vec::with_capacity(moves.len());
		macro_rules! gen_push_move {
			(dir $px:expr, $py:expr) => {
				|x, y, dir, signal| {
					let chunk_base_x = $px;
					let chunk_base_y = $py;
					let mov = Move::new((chunk_base_x + x, chunk_base_y + y), dir, signal);
					if !new_moves.contains(&mov) {
						new_moves.push(mov)
					}
				}
			};
			($px:expr, $py:expr) => {
				|x, y, signal| gen_push_move!(dir $px, $py)(x, y, Direction::from_rel((x, y)).map(|dir| dir.reverse()), signal)
			};
		}

		for mov in moves {
			let (x, y) = mov.to;
			let a = crate::continue_on_none!(self.at(x, y));

			*self.mut_at(x, y) = if let Some(b) = a.pass(mov.signal, mov.from, gen_push_move!(x, y))
			{
				b
			} else {
				*a
			};
		}
		for ((x, y), chunk) in &mut self.chunks {
			chunk.tick(gen_push_move!(
				dir * x * CHUNK_SIZE as i32,
				*y * CHUNK_SIZE as i32
			));
		}

		new_moves
	}

	fn ensure(&mut self, chunk_coords: (i32, i32)) {
		if !self.chunks.contains_key(&chunk_coords) {
			self.chunks.insert(chunk_coords, Chunk::default());
		}
	}
	pub fn at(&self, x: i32, y: i32) -> Option<&Block> {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.chunks
			.get(&chunk_coords)
			.map(|chunk| chunk.at(block_x, block_y))
			.flatten()
	}
	pub fn mut_at(&mut self, x: i32, y: i32) -> &mut Block {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.ensure(chunk_coords);
		self.chunks
			.get_mut(&chunk_coords)
			.unwrap_or_else(|| panic!("looks like World::ensure failed (world coords: {x} {y}, calculated chunk coords: {chunk_coords:?}, block coords: {block_x} {block_y})"))
			.mut_at(block_x, block_y).unwrap_or_else(|| panic!("chunk doesn't have coordinates block: {block_x} {block_y} @ world: {x} {y}"))
	}
	pub fn map_at(&mut self, x: i32, y: i32, f: impl FnOnce(Block) -> Block) {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.ensure(chunk_coords);
		self.chunks
			.get_mut(&chunk_coords)
			.map(|chunk| chunk.map_at(block_x, block_y, f));
	}

	pub fn chunks(&self) -> std::collections::hash_map::Iter<'_, (i32, i32), chunk::Chunk> {
		self.chunks.iter()
	}
}

pub fn world_coords_into_chunk_coords(x: i32, y: i32) -> ((i32, i32), (i32, i32)) {
	// this function is pure hell
	// i spent so much fucking time figuring this out i'm sure there's a one size fits all solution
	// for this but i didn't come up with it. here's the code(it works!) that calculates coords based on which
	// quarter of the world you're in

	if x >= 0 && y >= 0 {
		let chunk = (x / CHUNK_SIZE as i32, y / CHUNK_SIZE as i32);
		let block = (x % CHUNK_SIZE as i32, y % CHUNK_SIZE as i32);
		return (chunk, block);
	}
	if x < 0 && y >= 0 {
		let chunk = (x / CHUNK_SIZE as i32 - 1, y / CHUNK_SIZE as i32);
		let block = (
			(CHUNK_SIZE as i32 + (x % CHUNK_SIZE as i32)),
			// .min(CHUNK_SIZE as i32 - 1)
			// .max(0),
			y % CHUNK_SIZE as i32,
		);

		if block.0 == CHUNK_SIZE as i32 {
			return ((chunk.0 + 1, chunk.1), (0, block.1));
		}
		return (chunk, block);
	}
	if x >= 0 && y < 0 {
		let chunk = (x / CHUNK_SIZE as i32, y / CHUNK_SIZE as i32 - 1);
		let block = (
			x % CHUNK_SIZE as i32,
			(CHUNK_SIZE as i32 + (y % CHUNK_SIZE as i32)),
		);

		if block.1 == CHUNK_SIZE as i32 {
			return ((chunk.0, chunk.1 + 1), (block.0, 0));
		}
		return (chunk, block);
	}
	if x < 0 && y < 0 {
		let chunk = (x / CHUNK_SIZE as i32 - 1, y / CHUNK_SIZE as i32 - 1);
		let block = (
			(CHUNK_SIZE as i32 + (x % CHUNK_SIZE as i32)),
			(CHUNK_SIZE as i32 + (y % CHUNK_SIZE as i32)),
		);

		if block.0 == CHUNK_SIZE as i32 && block.1 != CHUNK_SIZE as i32 {
			return ((chunk.0 + 1, chunk.1), (0, block.1));
		}
		if block.1 == CHUNK_SIZE as i32 && block.0 != CHUNK_SIZE as i32 {
			return ((chunk.0, chunk.1 + 1), (block.0, 0));
		}
		if block.0 == CHUNK_SIZE as i32 && block.1 == CHUNK_SIZE as i32 {
			return ((chunk.0 + 1, chunk.1 + 1), (0, 0));
		}
		return (chunk, block);
	}

	todo!()
}
// pub fn chunk_coords_into_world_coords(
// 	(chunk_x, chunk_y): (i32, i32),
// 	(block_x, block_y): (i32, i32),
// ) -> (i32, i32) {
// 	(chunk_x + block_x, chunk_y + block_y)
// }

fn ensure_block_coords(a: i32) -> i32 {
	if a < 0 {
		let b = CHUNK_SIZE as i32 + a;
		return b;
	} else {
		a
	}
}
