use std::collections::HashMap;
use std::hash::Hash;

mod worlds;
use anyhow::{anyhow, Context};
pub use worlds::*;

use crate::{
	gfx::DrawType,
	world::{Block, Move, Signal, World},
};

// very proof of concept-y
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Game {
	pub drawmap: World<DrawType>,
	pub worlds: Worlds,
	/// the uuid of the world we have open \
	/// !!! can point to a world that doesn't exist so don't rely on that
	pub main_id: WorldId,

	/// moves of self.main
	pub moves: IngameWorld,
}
impl Game {
	pub fn tick(&mut self) -> anyhow::Result<()> {
		// reset the drawmap
		for (_, c) in self.drawmap.chunks_mut() {
			*c = Default::default();
		}

		if let Some(world) = self.worlds.at_mut(self.main_id) {
			let new_main_moves = world.tick(std::mem::take(&mut self.moves.moves), |x, y, dt| {
				*self.drawmap.mut_at(x, y) = dt
			});
			self.moves.process_moves(new_main_moves, |_mov| {
				// eprintln!("dropping a move returned from game.main ({_mov:?})")
			});
		}

		let mut moves = std::mem::take(&mut self.moves);
		moves.tick_children(self)?;
		self.moves = moves;
		Ok(())
	}

	/// creates a new world, returning its id
	pub fn push(&mut self) -> WorldId {
		self.worlds.push(Default::default())
	}

	pub fn switch_main(&mut self, id: WorldId) {
		self.main_id = id;
	}

	pub fn main(&self) -> Option<&World> {
		self.worlds.at(self.main_id)
	}
	pub fn worlds(&self) -> impl Iterator<Item = &World> {
		self.worlds.iter().map(|(_, w)| w)
	}
}
impl Hash for Game {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		// drawmap is excluded on purpose
		self.main_id.hash(state);
		self.moves.hash(state);
		self.worlds.hash(state);
	}
}

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
// so long Game struct (it stayed)

/// IngameWorld is a recursive structure that contains the moves (instance) of the world it's pointing to in id \
/// this is needed to make every world have unique instances of worlds, to be contained in foreigns
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IngameWorld {
	pub world_id: WorldId,
	pub moves: Vec<Move>,
	pub children: Vec<IngameWorld>,
}
impl IngameWorld {
	/// generates the ingameworld required for a World to have foreigns working \
	/// see [IngameWorld::regenerate]
	pub fn generate(game: &mut Game, world_id: WorldId) -> Self {
		let mut ingameworld = Self::default();
		ingameworld.regenerate(game, world_id);
		ingameworld
	}
	/// regenerates itself and fixes the world if needed \
	/// recursive
	pub fn regenerate(&mut self, game: &mut Game, world_id: WorldId) -> anyhow::Result<()> {
		let world = match game.worlds.at(world_id) {
			Some(a) => a,
			None => {
				return Err(anyhow!(
					"could not find a world with id {world_id}\nneeded for IngameWorld::regenerate"
				))
			}
		};
		let mut foreigns =
			game.worlds
				.at(world_id)
				.with_context(|| {
					format!("could not find world with id {world_id}\nneeded to regenerate an IngameWorld")
				})?
				.find_foreigns();
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
							game.worlds
								.at_mut(world_id)
								.with_context(|| "impossible case in IngameWorld::regenerate")?
								.map_at(coords.0, coords.1, |_| {
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
					game.worlds
						.at_mut(world_id)
						.with_context(|| "second impossible case in IngameWorld::regenerate")?
						.map_at(coords.0, coords.1, |_| {
							Block::Error("here lies a foreign to this world (infinite recursion)")
						});
					continue;
				}

				inst.regenerate(game, inst_world_id);
				inst_ids_already_done.push(inst_id);
			}

			let inst_world = match game.worlds.at(inst_world_id) {
				Some(a) => a,
				None => {
					eprintln!("deleting a foreign referencing an invalid world ({inst_world_id}) in {world_id}");
					// this foreign has an invalid world id
					*game
						.worlds
						.at_mut(world_id)
						.with_context(|| "impossible case 3 in IngameWorld::regenerate()")?
						.mut_at(coords.0, coords.1) = Block::Error(
						"here lies a foreign that pointed to a world that doesn't exist",
					);
					return Ok(()); // <- fake Ok
				}
			};

			let next = match next_id_per_inst_id.get(&inst_id) {
				Some(next) => *next,
				None => 0,
			};
			let max_id = inst_world.inputs_count().max(inst_world.outputs_count());
			let world_mut = game
				.worlds
				.at_mut(world_id)
				.with_context(|| "impossible case 3 IngameWorld::regenerate()")?;
			if id > max_id {
				eprintln!("the world (id: {world_id:?}) contained a foreign that exceeded the maximum possible id of {max_id} for the world given ({inst_world_id:?}) by being {id}");
				world_mut.map_at(coords.0, coords.1, |_| {
					Block::Error("here lies a foreign that exceeded the maximum possible id for the world given")
				});
			} else {
				world_mut.map_at(coords.0, coords.1, |_| {
					Block::Foreign(inst_world_id, inst_id, next)
				});
				next_id_per_inst_id.insert(inst_id, next + 1);
			}
		}
		Ok(())
	}

	fn tick(&mut self, game: &mut Game, ret: impl FnMut(Move)) -> anyhow::Result<()> {
		let new_moves = game
			.worlds
			.at_mut(self.world_id)
			.with_context(|| format!("this IngameWorld points to a nonexistent world\n{self:#?}"))?
			.tick(std::mem::take(&mut self.moves), |_, _, _| {});
		self.process_moves(new_moves, ret);
		Ok(())
	}
	fn tick_children(&mut self, game: &mut Game) -> anyhow::Result<()> {
		for (i, child) in self.children.iter_mut().enumerate() {
			child.tick(game, |m| match m {
				Move::Output { id, .. } => self.moves.push(Move::Foreign {
					inst_id: i,
					id,
					signal: Signal::ExternalPoweron,
				}),
				mov => eprintln!("only outputs should be returned from child worlds ({mov:?})"),
			})?;
			child.tick_children(game);
		}
		Ok(())
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

		let push_unique = |moves: &mut Vec<_>, mov: Move| {
			if !moves.contains(&mov) {
				moves.push(mov);
			}
		};

		for mov in new_moves {
			match mov {
				Move::Inside { .. } => push_unique(&mut self.moves, mov),
				Move::Output { .. } => ret(mov),
				Move::Foreign { inst_id, id, .. } => push_unique(
					&mut self.child_mut(inst_id).moves,
					Move::Input {
						id,
						signal: Signal::ExternalPoweron,
					},
				),
				Move::Input { .. } => {
					eprintln!("unexpected input move in moves processing: {mov:?}")
				}
			}
		}
	}
}
