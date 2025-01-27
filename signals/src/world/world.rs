use serde::{Deserialize, Serialize};

use crate::{game::WorldId, gfx, world::*};

use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum Signal {
	#[default]
	Default,
	ExternalPoweron,
	DefaultIf(fn(Block) -> bool),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Move {
	Inside {
		to: (i32, i32),
		from: Option<Direction>,
		signal: Signal,
	},
	/// inputs are placed into the moves vec by main, not world
	Input { id: usize, signal: Signal },
	/// outputs are placed in the vec by world but are supposed to be handled externally
	Output { id: usize, signal: Signal },
	/// foreigns
	Foreign {
		inst_id: usize,
		id: usize,
		signal: Signal,
	},
}
impl Move {
	// wth i'm gonna have a new function
	#[allow(dead_code)]
	pub fn new(to: (i32, i32), from: Option<Direction>, signal: Signal) -> Self {
		Self::Inside { to, from, signal }
	}

	pub fn signal(&self) -> &Signal {
		match self {
			Move::Inside { signal, .. } => signal,
			Move::Input { signal, .. } => signal,
			Move::Output { signal, .. } => signal,
			Move::Foreign { signal, .. } => signal,
		}
	}
}
/// the to field of the push_move function
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PushMoveTo {
	Rel(i32, i32),
	OutputID(usize),
	Foreign { inst_id: usize, id: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct World<B = Block> {
	chunks: HashMap<(i32, i32), Chunk<B>>,
}
impl<B> World<B> {
	pub fn at(&self, x: i32, y: i32) -> Option<&B> {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.chunks
			.get(&chunk_coords)
			.map(|chunk| chunk.at(block_x, block_y))
			.flatten()
	}
	#[allow(unused)]
	pub fn set_chunk(&mut self, coords: (i32, i32), chunk: Chunk<B>) -> Option<Chunk<B>> {
		self.chunks.insert(coords, chunk)
	}
	pub fn chunk(&self, coords: (i32, i32)) -> Option<&Chunk<B>> {
		self.chunks.get(&coords)
	}
}
impl<B: Default> World<B> {
	fn ensure(&mut self, chunk_coords: (i32, i32)) {
		if !self.chunks.contains_key(&chunk_coords) {
			self.chunks.insert(chunk_coords, Chunk::default());
		}
	}
	pub fn mut_at(&mut self, x: i32, y: i32) -> &mut B {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.ensure(chunk_coords);
		self.chunks
			.get_mut(&chunk_coords)
			.unwrap_or_else(|| panic!("looks like World::ensure failed (world coords: {x} {y}, calculated chunk coords: {chunk_coords:?}, block coords: {block_x} {block_y})"))
			.mut_at(block_x, block_y).unwrap_or_else(|| panic!("chunk doesn't have coordinates block: {block_x} {block_y} @ world: {x} {y}"))
	}
	pub fn chunks(&self) -> std::collections::hash_map::Iter<'_, (i32, i32), chunk::Chunk<B>> {
		self.chunks.iter()
	}
	pub fn chunks_mut(
		&mut self,
	) -> std::collections::hash_map::IterMut<'_, (i32, i32), chunk::Chunk<B>> {
		self.chunks.iter_mut()
	}

	/// returns: ((x, y) size in chunks, (x, y) chunk offset added to base coords to make world render the left upmost chunk at 0,0)
	pub fn size_and_offset(&self) -> ((i32, i32), (i32, i32)) {
		let ((x_smallest, x_biggest), (y_smallest, y_biggest)) = match self.biggest_smallest() {
			None => return ((0, 0), (0, 0)),
			Some(a) => a,
		};

		let x_offset = -x_smallest;
		let y_offset = -y_smallest;

		(
			(x_biggest - x_smallest + 1, y_biggest - y_smallest + 1),
			(x_offset, y_offset),
		)
	}
	/// returns none if there are no chunks
	pub fn biggest_smallest(&self) -> Option<((i32, i32), (i32, i32))> {
		let (mut x_smallest, mut x_biggest) = (0, 0);
		let (mut y_smallest, mut y_biggest) = (0, 0);

		let mut did_anything = false;

		for ((cx, cy), _) in self.chunks() {
			did_anything = true;

			x_smallest = x_smallest.min(*cx);
			x_biggest = x_biggest.max(*cx);

			y_smallest = y_smallest.min(*cy);
			y_biggest = y_biggest.max(*cy);
		}

		if did_anything {
			Some(((x_smallest, x_biggest), (y_smallest, y_biggest)))
		} else {
			None
		}
	}

	/// returns an iterator over every block there is in the world
	pub fn blocks(&self) -> impl Iterator<Item = ((i32, i32), &B)> {
		self.chunks()
			.map(|(cc, c)| {
				c.blocks_with_coords()
					.map(|(bc, b)| (chunk_coords_into_world_coords(*cc, bc), b))
			})
			.flatten()
	}
}
impl<B: Default + Copy> World<B> {
	pub fn map_at(&mut self, x: i32, y: i32, f: impl FnOnce(B) -> B) {
		let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
		self.ensure(chunk_coords);
		self.chunks
			.get_mut(&chunk_coords)
			.map(|chunk| chunk.map_at(block_x, block_y, f));
	}
}
impl<B: Default + PartialEq> World<B> {
	#[allow(unused)]
	pub fn is_block_at(&self, x: i32, y: i32) -> bool {
		match self.at(x, y) {
			None => false,
			Some(a) if *a != B::default() => true,
			_ => false,
		}
	}
}
impl World<Block> {
	/// set_drawtype should handle junctions where for all axis sets it to previous_value || new_value \
	/// just call [gfx::DrawType::apply_new]
	pub fn tick<D: FnMut(i32, i32, gfx::DrawType)>(
		&mut self,
		moves: Vec<Move>,
		mut set_drawtype: D,
	) -> Vec<Move> {
		let mut new_moves = Vec::with_capacity(moves.len());
		macro_rules! gen_push_move {
			($x:expr, $y:expr) => {
				|to, signal| match to {
					PushMoveTo::Rel(rx, ry) => {
						let to = ($x + rx, $y + ry);
						new_moves.push(Move::Inside {
							to,
							from: Direction::from_rel((rx, ry)).map(|dir| dir.reverse()),
							signal,
						});
					}
					PushMoveTo::OutputID(id) => new_moves.push(Move::Output { id, signal }),
					PushMoveTo::Foreign { inst_id, id } => new_moves.push(Move::Foreign {
						inst_id,
						id,
						signal,
					}),
				}
			};
		}

		for mov in moves.into_iter().filter_map(|mov| match mov.signal() {
			Signal::DefaultIf(f) => match mov {
				Move::Inside { to,  from, .. } => {
					if self.at(to.0, to.1).map(|b| f(*b)).unwrap_or(false) {
						Some(Move::Inside { to, from, signal: Signal::Default })
					} else {
						None
					}
				}
				_ => {
					eprintln!("dropping a Signal::DefaultIf, since it only works with Move::Inside for performance");
					None
				}
			},
			_ => Some(mov)
		}).collect::<Vec<_>>() {
			match mov {
				Move::Inside { to, from, signal } => {
					let (x, y) = to;
					let a = *crate::continue_on_none!(self.at(x, y));

					// if a is a wire receiving a signal from the direction it's passing signals
					match a {
						Block::Wire(dir) => {
						if !from.map(|from| from == dir).unwrap_or(false) {
						set_drawtype(to.0, to.1, gfx::DrawType::On);
						}

						}
						Block::Junction => {
							if let Some(dir) = from {
								match dir {
									Direction::Bottom | Direction::Top => set_drawtype(to.0, to.1, gfx::DrawType::Junction { vertical: true, horizontal: false }),
									Direction::Right | Direction::Left => set_drawtype(to.0, to.1, gfx::DrawType::Junction { vertical: false, horizontal: true })
								}
							}
						}
						_ => set_drawtype(to.0, to.1, gfx::DrawType::On)
					}

					*self.mut_at(x, y) = if let Some(b) = a.pass(signal, from, gen_push_move!(x, y))
					{
						b
					} else {
						a
					};
				}
				Move::Input { id, signal } => {
					let input = match self.find_input(id) {
						Some(a) => a,
						None => {
							eprintln!("didn't find an input with id {id}, dropping {signal:?}");
							continue;
						}
					};
					let block = self
						.at(input.0, input.1)
						.expect("couldn't find the block world.find_input found");
					*self.mut_at(input.0, input.1) = if let Some(b) =
						block.pass(signal, None, gen_push_move!(input.0, input.1))
					{
						b
					} else {
						*block
					}
				}
				Move::Foreign {  inst_id, id, signal } => {
					let input = match self.find_foreign(inst_id, id) {
						Some(a) => a,
						None => {
							eprintln!("didn't find an foreign with inst_id {inst_id} and id {id}, dropping {signal:?}");
							continue;
						}
					};
					let block = self
						.at(input.0, input.1)
						.expect("couldn't find the block world.find_input found");
					*self.mut_at(input.0, input.1) = if let Some(b) =
						block.pass(signal, None, gen_push_move!(input.0, input.1))
					{
						b
					} else {
						*block
					}
				},
				Move::Output { id, signal } => eprintln!("no Move::Output variant should be in the moves vec sent to world.tick, (Move::Output {{ id: {id}, signal: {signal:?} }})")
			}
		}
		for ((x, y), chunk) in &mut self.chunks {
			// chunk.tick(gen_push_move!(
			// 	dir * x * CHUNK_SIZE as i32,
			// 	*y * CHUNK_SIZE as i32
			// ));
			chunk.tick(|to, (bx, by), signal| match to {
				PushMoveTo::Rel(rx, ry) => {
					let to = (
						(*x) * CHUNK_SIZE as i32 + bx + rx,
						(*y) * CHUNK_SIZE as i32 + by + ry,
					);
					new_moves.push(Move::Inside {
						to,
						from: Direction::from_rel((rx, ry)).map(|dir| dir.reverse()),
						signal,
					});
				}
				PushMoveTo::OutputID(id) => new_moves.push(Move::Output { id, signal }),
				PushMoveTo::Foreign { inst_id, id } => new_moves.push(Move::Foreign {
					inst_id,
					id,
					signal,
				}),
			})
		}

		new_moves.dedup();
		new_moves
	}

	/// returns worlds coords
	pub fn find_input(&self, id: usize) -> Option<(i32, i32)> {
		for (coords, c) in self.chunks() {
			for x in 0..CHUNK_SIZE as i32 {
				for y in 0..CHUNK_SIZE as i32 {
					// this implementation will halt searching as soon as a matching one is found, might lead to weird behavior with
					// duplicate ids
					match c.at(x, y) {
						None => panic!(
							"a number between 0 and CHUNK_SIZE shouldn't be larger than CHUNK_SIZE"
						),
						Some(Block::Input(i_id)) => {
							if *i_id == id {
								return Some(chunk_coords_into_world_coords(*coords, (x, y)));
							}
						}
						_ => continue,
					}
				}
			}
		}
		None
	}
	pub fn find_foreign(&self, inst_id: usize, id: usize) -> Option<(i32, i32)> {
		for (coords, c) in self.chunks() {
			for x in 0..CHUNK_SIZE as i32 {
				for y in 0..CHUNK_SIZE as i32 {
					// this implementation will halt searching as soon as a matching one is found, might lead to weird behavior with
					// duplicate ids
					match c.at(x, y) {
						None => panic!(
							"a number between 0 and CHUNK_SIZE shouldn't be larger than CHUNK_SIZE"
						),
						Some(Block::Foreign(_, b_iid, b_id)) => {
							if *b_iid == inst_id && *b_id == id {
								return Some(chunk_coords_into_world_coords(*coords, (x, y)));
							}
						}
						_ => continue,
					}
				}
			}
		}
		None
	}
	pub fn find_foreigns<'a>(
		&'a self,
	) -> impl Iterator<Item = ((i32, i32), (WorldId, usize, usize))> + 'a {
		self.blocks().filter_map(|(coords, b)| match b {
			Block::Foreign(wid, inst_id, id) => Some((coords, (*wid, *inst_id, *id))),
			_ => None,
		})
	}

	pub fn inputs_count(&self) -> usize {
		let mut i = 0;

		for _ in self
			.chunks()
			.map(|(_, c)| c.blocks())
			.flatten()
			.filter(|b| match b {
				Block::Input(_) => true,
				_ => false,
			}) {
			i += 1;
		}

		i
	}
	pub fn outputs_count(&self) -> usize {
		let mut i = 0;

		for _ in self
			.chunks()
			.map(|(_, c)| c.blocks())
			.flatten()
			.filter(|b| match b {
				Block::Output(_) => true,
				_ => false,
			}) {
			i += 1;
		}

		i
	}

	/// this makes sure input-output block's ids are in order
	/// and there are no holes or duplicates\
	/// returns (inputs.len + 1, outputs.len + 1)
	pub fn io_blocks_fix(&mut self) -> (usize, usize) {
		let mut inputs = vec![];
		let mut outputs = vec![];

		for (coords, c) in self.chunks() {
			for x in 0..CHUNK_SIZE as i32 {
				for y in 0..CHUNK_SIZE as i32 {
					let block = c.at(x, y).expect("this should work");

					match block {
						Block::Input(id) => {
							inputs.push((*id, chunk_coords_into_world_coords(*coords, (x, y))))
						}
						Block::Output(id) => {
							outputs.push((*id, chunk_coords_into_world_coords(*coords, (x, y))))
						}
						_ => continue,
					}
				}
			}
		}

		inputs.sort_by_key(|a| a.0);
		outputs.sort_by_key(|a| a.0);

		let mut in_i = 0;
		let mut out_i = 0;

		for (i, (id, (x, y))) in inputs.iter().copied().enumerate() {
			if i != id {
				match self.mut_at(x, y) {
					Block::Input(live_id) => *live_id = i,
					_ => eprintln!("this case is literally impossible"),
				}
			}
			in_i = i;
		}
		for (i, (id, (x, y))) in outputs.iter().copied().enumerate() {
			if i != id {
				match self.mut_at(x, y) {
					Block::Output(live_id) => *live_id = i,
					_ => eprintln!("this case is literally impossible"),
				}
			}
			out_i = i;
		}

		(in_i, out_i)
	}

	pub fn outputs<'a>(&'a self) -> impl Iterator<Item = (usize, (i32, i32))> + 'a {
		self.blocks().filter_map(|(c, b)| match b {
			Block::Output(id) => Some((*id, c)),
			_ => None,
		})
	}
}
impl Hash for World {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for (coords, c) in self.chunks() {
			coords.hash(state);
			c.hash(state);
		}
	}
}

pub fn chunk_coords_into_world_coords(
	(chunk_x, chunk_y): (i32, i32),
	(block_x, block_y): (i32, i32),
) -> (i32, i32) {
	(
		chunk_x * CHUNK_SIZE as i32 + block_x,
		chunk_y * CHUNK_SIZE as i32 + block_y,
	)
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
