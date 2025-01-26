use crate::{
	game::WorldId,
	world::{Block, Direction, World},
};

/// this is just the pattern the start coords and the foreigns inside the clump generate
pub fn clump_pattern((start_x, start_y): (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
	// i want to make it so that a one block gap is okay
	// so the area we need to scan:
	//     x
	//   x x x
	// x x   x x
	//   x x x
	//     x
	// i'm gonna do all this with iterators

	let make_line = |dir, rng: std::ops::Range<i32>| rng.map(move |a| (dir, a));
	let make_line = move |dir| make_line(dir, 1..3);

	let lines = Direction::all().map(make_line).flatten();
	let lines = lines.map(|(dir, mul)| dir.rel_mul(mul));
	let to_scan = lines.chain([(-1, -1), (-1, 1), (1, -1), (1, 1)]);

	let to_scan = to_scan.map(move |(rx, ry)| (start_x + rx, start_y + ry));
	to_scan
}

/// FindClump provides World::find_clump to find foreign clumps
pub trait FindClump {
	/// see [Clump] and the comments in [clump_pattern]
	fn find_clump(&self, start_coords: (i32, i32)) -> Clump;
}
impl FindClump for World {
	fn find_clump(&self, start_coords: (i32, i32)) -> Clump {
		Clump {
			world: self,
			i: 0,
			queue: clump_pattern(start_coords).collect(),
		}
	}
}

/// Clump is an iterator that yields every coordinate in a given foreign clump
#[warn(unused)]
#[derive(Clone, Debug)]
pub struct Clump<'a> {
	world: &'a World,

	queue: Vec<(i32, i32)>,
	i: usize,
}
impl<'a> Clump<'a> {
	/// turn self (iterator of coords) into iterator of (wid, inst_id, id)
	pub fn foreign_data(self) -> impl Iterator<Item = (WorldId, usize, usize)> + 'a {
		let world = self.world;
		let clump = self.filter_map(|(x, y)| match world.at(x, y) {
			Some(&Block::Foreign(wid, inst_id, id)) => Some((wid, inst_id, id)),
			_ => None,
		});
		clump
	}

	fn push_unique_to_queue<I: IntoIterator<Item = (i32, i32)>>(&mut self, iter: I) {
		let iter = iter.into_iter();

		self.queue.reserve(iter.size_hint().0);
		for coords in iter {
			if !self.queue.contains(&coords) {
				self.queue.push(coords);
			}
		}
	}

	fn add_pattern_to_queue(&mut self, base_coords: (i32, i32)) {
		let pattern = clump_pattern(base_coords);
		self.push_unique_to_queue(pattern);
	}
	fn next_foreign_in_queue(&mut self) -> Option<(i32, i32)> {
		loop {
			let top = self.queue.iter().nth(self.i).copied()?;
			self.i += 1;
			let b = match self.world.at(top.0, top.1) {
				Some(a) => a,
				None => continue,
			};
			match b {
				Block::Foreign(_, _, _) => return Some(top),
				_ => continue,
			}
		}
	}
}
impl<'a> Iterator for Clump<'a> {
	type Item = (i32, i32);
	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.queue.len() - self.i))
	}

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(f_coords) = self.next_foreign_in_queue() {
			let pattern = clump_pattern(f_coords);
			self.push_unique_to_queue(pattern);

			Some(f_coords)
		} else {
			None
		}
	}
}
