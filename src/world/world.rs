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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
	chunks: HashMap<(i32, i32), Chunk>,
}
impl Default for World {
	fn default() -> Self {
		World {
			chunks: [((0, 0), Chunk::checkerboard())].into_iter().collect(),
		}
	}
}
impl World {
	pub fn tick(&mut self, moves: Vec<Move>) -> Vec<Move> {
		let mut new_moves = Vec::new();
		macro_rules! gen_push_move {
			($px:expr, $py:expr) => {
				|x, y, signal| {
					let mov = Move::new(
						($px + x, $py + y),
						Direction::from_rel((x, y)).map(|dir| dir.reverse()),
						signal,
					);
					if !new_moves.contains(&mov) {
						new_moves.push(mov)
					}
				}
			};
		}

		for mov in moves {
			let (x, y) = mov.to;
			let a = crate::continue_on_none!(self.at(x, y));

			*crate::continue_on_none!(self.mut_at(x, y)) =
				if let Some(b) = a.pass(mov.signal, mov.from, gen_push_move!(x, y)) {
					b
				} else {
					*a
				};
		}
		for ((x, y), chunk) in &mut self.chunks {
			chunk.tick(gen_push_move!(*x, *y));
		}

		new_moves
	}

	pub fn at(&self, x: i32, y: i32) -> Option<&Block> {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.chunks
			.get(&chunk_coords)
			.map(|chunk| chunk.at(block_x, block_y))
			.flatten()
	}
	pub fn mut_at(&mut self, x: i32, y: i32) -> Option<&mut Block> {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.chunks
			.get_mut(&chunk_coords)
			.map(|chunk| chunk.mut_at(block_x, block_y))
			.flatten()
	}
	pub fn map_at(&mut self, x: i32, y: i32, f: impl FnOnce(Block) -> Block) {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.chunks
			.get_mut(&chunk_coords)
			.map(|chunk| chunk.map_at(block_x, block_y, f));
	}

	pub fn draw_at(&self, d: &mut RaylibDrawHandle, base_x: i32, base_y: i32) {
		for (coords, chunk) in &self.chunks {
			chunk.draw_at(
				d,
				base_x + coords.0 * CHUNK_SIZE as i32 * BLOCK_SIZE as i32,
				base_y + coords.1 * CHUNK_SIZE as i32 * BLOCK_SIZE as i32,
			);
		}
	}
}

pub fn world_coords_into_chunk_coords(x: i32, y: i32) -> ((i32, i32), (i32, i32)) {
	let chunk_coords = (x % CHUNK_SIZE as i32, y % CHUNK_SIZE as i32);
	let block_coords = (x / CHUNK_SIZE as i32, y / CHUNK_SIZE as i32);
	(chunk_coords, block_coords)
}
pub fn chunk_coords_into_world_coords(
	(chunk_x, chunk_y): (i32, i32),
	(block_x, block_y): (i32, i32),
) -> (i32, i32) {
	(chunk_x + block_x, chunk_y + block_y)
}
