use std::collections::HashMap;

use crate::world::{Block, Move, RenderedWorld, Signal, World};

// how are we gonna get a whole world inside a block? i hear you asking
// create a block that contains: world id, input id, output id
// for now we can just hardcode for example signals from the left will be treated as input,
// while all other sides will output the output
// this is actually feasible but i'm soooooo tired gn

// we need a new architecture this one sucks ass
// ingame world:
// - World
// - moves
// - children (Vec<ingame world>)

// this makes everything easier let's implement it
// so long Game struct

/// IngameWorld is a recursive structure that contains the moves (instance) of the world it's pointing to in id \
/// this is needed to make every world have unique instances of worlds, to be contained in foreigns
#[derive(Clone, Debug, Default)]
pub struct IngameWorld {
	/// index of game, see Game::world
	pub world_id: Option<usize>,
	pub moves: Vec<Move>,
	pub children: Vec<IngameWorld>,
}
impl IngameWorld {
	/// generates the ingameworld required for a World to have foreigns working \
	/// see [IngameWorld::regenerate]
	pub fn generate(game: &mut Game, world_id: Option<usize>) -> Self {
		let mut ingameworld = Self::default();
		ingameworld.regenerate(game, world_id);
		ingameworld
	}
	/// regenerates itself and fixes the world if needed \
	/// recursive
	pub fn regenerate(&mut self, game: &mut Game, world_id: Option<usize>) {
		let mut foreigns = game.world(world_id).find_foreigns();
		foreigns.sort_by(|(_, (_, a_inst_id, a_id)), (_, (_, b_inst_id, b_id))| {
			(a_inst_id * 1000 + a_id).cmp(&(b_inst_id * 1000 + b_id))
		});

		let mut inst_ids_already_done =
			Vec::with_capacity((self.children.len() as i32 - 1 as i32).max(0) as usize);
		let mut next_id_per_inst_id: HashMap<usize, usize> = HashMap::new();

		for (coords, (inst_world_id, mut inst_id, id)) in foreigns {
			if !inst_ids_already_done.contains(&inst_id) {
				// only happens once per inst_id
				let inst = match self.children.iter_mut().nth(inst_id) {
					Some(ptr) => ptr,
					None => {
						if inst_id != self.children.len() {
							inst_id = self.children.len();
							game.world_mut(world_id).map_at(coords.0, coords.1, |_| {
								Block::Foreign(inst_world_id, inst_id, id)
							})
						}
						self.children.push(IngameWorld {
							world_id: inst_world_id,
							..Default::default()
						});
						&mut self.children[inst_id]
					}
				};

				if inst_world_id == world_id {
					println!("deleting a foreign referencing the world the foreign's in (world_id: {world_id:?})");
					game.world_mut(world_id).map_at(coords.0, coords.1, |_| {
						Block::Error("here lies a foreign to this world (infinite recursion)")
					});
					continue;
				}

				inst.regenerate(game, inst_world_id);
				inst_ids_already_done.push(inst_id);
			}

			let inst_world = game.world(inst_world_id);

			let next = match next_id_per_inst_id.get(&inst_id) {
				Some(next) => *next,
				None => 0,
			};
			let max_id = inst_world.inputs_count().max(inst_world.outputs_count());
			if id > max_id {
				eprintln!("the world (id: {world_id:?}) contained a foreign that exceeded the maximum possible id of {max_id} for the world given ({inst_world_id:?}) by being {id}");
				game.world_mut(world_id).map_at(coords.0, coords.1, |_| {
					Block::Error("here lies a foreign that exceeded the maximum possible id for the world given")
				});
			} else {
				game.world_mut(world_id).map_at(coords.0, coords.1, |_| {
					Block::Foreign(inst_world_id, inst_id, next)
				});
				next_id_per_inst_id.insert(inst_id, next + 1);
			}
		}
	}

	fn tick(&mut self, game: &mut Game, ret: impl FnMut(Move)) {
		let new_moves = game
			.world_mut(self.world_id)
			.tick(std::mem::take(&mut self.moves), |_, _, _| {});
		self.process_moves(new_moves, ret)
	}
	fn tick_children(&mut self, game: &mut Game) {
		for (i, child) in self.children.iter_mut().enumerate() {
			child.tick(game, |m| match m {
				Move::Output { id, .. } => self.moves.push(Move::Foreign {
					inst_id: i,
					id,
					signal: Signal::ForeignExternalPoweron,
				}),
				mov => eprintln!("only outputs should be returned from child worlds ({mov:?})"),
			});
			child.tick_children(game);
		}
	}

	fn child_mut(&mut self, inst_id: usize) -> &mut Self {
		if self.children.len() as i32 - 1 <= inst_id as i32 {
		} else {
			let to_add = inst_id as i32 - self.children.len() as i32 + 1;
			for _ in 0..to_add {
				self.children.push(Default::default());
			}
		}
		&mut self.children[inst_id]
	}

	fn process_moves(&mut self, new_moves: Vec<Move>, mut ret: impl FnMut(Move)) {
		self.moves = Vec::with_capacity(new_moves.len());

		for mov in new_moves {
			match mov {
				Move::Inside { .. } => self.moves.push(mov),
				Move::Output { .. } => ret(mov),
				Move::Foreign {
					inst_id,
					id,
					signal, // we assume this isn't a ForeignExternalPoweron cause how'd that end up in new_moves
				} => self
					.child_mut(inst_id)
					.moves
					.push(Move::Input { id, signal }),
				Move::Input { .. } => {
					eprintln!("unexpected input move in moves processing: {mov:?}")
				}
			}
		}
	}
}

// very proof of concept-y
#[derive(Clone, Debug, Default)]
pub struct Game {
	// the way this works is when we switch to another world we write the index of
	// the world we switched to in i, when we switch back we read i to figure out worlds[?] is main
	pub main: RenderedWorld,
	worlds: Vec<World>,
	pub i: Option<usize>,

	/// moves of self.main
	pub moves: IngameWorld,
}
impl Game {
	pub fn tick(&mut self) {
		let new_main_moves = self.main.tick(std::mem::take(&mut self.moves.moves));
		self.moves.process_moves(new_main_moves, |mov| {
			eprintln!("dropping a move returned from game.main ({mov:?})")
		});

		let mut moves = std::mem::take(&mut self.moves);
		moves.tick_children(self);
		self.moves = moves;
	}

	/// creates a new worlds, returning its id
	pub fn push(&mut self) -> Option<usize> {
		self.worlds.push(Default::default());
		Some(self.worlds.len() - 1)
	}
	/// switch main to `new_i`
	pub fn switch_main(&mut self, new_i: Option<usize>) {
		// grow the worlds vec if neccessary
		if let Some(new_i) = new_i {
			if self.worlds.len() <= new_i {
				self.worlds.resize_with(new_i + 1, Default::default);
			}
		}

		let mut switch_main_with = |i: usize| {
			use std::mem;

			let old_main = mem::take(&mut self.main).take();
			let world_at_i = mem::replace(&mut self.worlds[i], old_main);

			self.main = RenderedWorld::new(world_at_i);
		};

		// reset main
		match self.i {
			Some(i) => {
				switch_main_with(i);
				self.i = None;
			}
			None => {}
		};

		if let Some(new_i) = new_i {
			switch_main_with(new_i);
			self.i = Some(new_i);
		}
	}

	/// returns main on None, world number n at Some(n), even if some numbered world is switched with main
	pub fn world(&mut self, id: Option<usize>) -> &World {
		// grow the worlds vec if neccessary
		if let Some(new_i) = id {
			if self.worlds.len() <= new_i {
				self.worlds.resize_with(new_i + 1, Default::default);
			}
		}

		match id {
			None => match self.i {
				None => self.main.as_ref(),
				Some(wh) => &self.worlds[wh],
			},
			Some(a) => {
				if self.i != Some(a) {
					&self.worlds[a]
				} else {
					self.main.as_ref()
				}
			}
		}
	}
	/// see Self::world
	pub fn world_mut(&mut self, id: Option<usize>) -> &mut World {
		// grow the worlds vec if neccessary
		if let Some(new_i) = id {
			if self.worlds.len() <= new_i {
				self.worlds.resize_with(new_i + 1, Default::default);
			}
		}

		match id {
			None => match self.i {
				None => self.main.as_mut(),
				Some(wh) => &mut self.worlds[wh],
			},
			Some(a) => {
				if self.i != Some(a) {
					&mut self.worlds[a]
				} else {
					self.main.as_mut()
				}
			}
		}
	}

	/// [Self::world], but will not create the world if a world with `id` doesn't exist
	pub fn world_opt(&self, id: Option<usize>) -> Option<&World> {
		match id {
			None => match self.i {
				None => Some(self.main.as_ref()),
				Some(wh) => self.worlds.iter().nth(wh),
			},
			Some(a) => {
				if self.i != Some(a) {
					self.worlds.iter().nth(a)
				} else {
					Some(self.main.as_ref())
				}
			}
		}
	}

	pub fn worlds(&self) -> impl Iterator<Item = &World> {
		let main = std::iter::once(self.world_opt(None));
		let numbered = (0..self.worlds.len().max(1) - 1).map(|i| self.world_opt(Some(i)));

		main.chain(numbered)
			.map(|opt| opt.expect("Game::worlds failed, as one of the worlds returned is None"))
	}
}
