use std::{collections::HashMap, hash::Hash};

mod worlds;
use anyhow::Context;
pub use worlds::*;

mod ingameworld;
pub use ingameworld::*;

pub mod saves;

use crate::{gfx::DrawType, processor, world::World};

// ok so foreigns are great but we have a processor system we need to implement
// game should store the memory and the programs and take care of regenerating them as needed,
// while ingameworld should have the cache of the previous inputs and the output it generated.
// if the inputs change, rerun the program and shit

// very proof of concept-y
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Game {
	pub worlds: Worlds,

	/// the uuid of the world we have open \
	/// !!! can point to a world that doesn't exist so don't rely on that
	pub main_id: WorldId,
	pub drawmap: World<DrawType>,

	/// moves of self.main
	pub moves: IngameWorld,

	memory: processor::Memory,
	pub programs: Programs,
}
type Programs = HashMap<WorldId, (Option<Vec<processor::Instruction>>, usize, usize)>; // v: (none if errored during instgen, inputs_len, outputs_len)
impl Game {
	pub fn from_worlds(worlds: Worlds) -> anyhow::Result<Self> {
		// since the world loads with a nonexistent main_id, it's ok to just use Default::default()
		// for everything since as soon as we switch to something everything that needs to be generated
		// will be generated

		let wids = worlds.iter().map(|(a, _)| a).copied().collect::<Vec<_>>();
		let mut game = Self {
			worlds,
			..Default::default()
		};
		for uuid in wids {
			game.generate_program_for(uuid)
				.with_context(|| "in Game::from_worlds")?;
		}
		Ok(game)
	}

	pub fn tick(&mut self) -> anyhow::Result<()> {
		// reset the drawmap
		for (_, c) in self.drawmap.chunks_mut() {
			*c = Default::default();
		}

		if let Some(world) = self.worlds.at_mut(self.main_id) {
			let mut taken_moves = std::mem::take(&mut self.moves);
			taken_moves.tick(
				self,
				|_mov| {
					// eprintln!("dropping a move returned from game.main ({_mov:?})")
				},
				true,
			);

			taken_moves.tick_children(self)?;
			self.moves = taken_moves;
		}
		Ok(())
	}

	/// creates a new world, returning its id
	pub fn push(&mut self) -> WorldId {
		self.worlds.push(Default::default())
	}

	pub fn switch_main(&mut self, id: WorldId) {
		let prev_id = self.main_id;
		self.main_id = id;
		if let Err(err) = self.regenerate_moves(prev_id) {
			eprintln!("error while regenerating moves after Game::switch_main call\n{err}")
		}
		if let Err(err) = self.generate_programs_for(prev_id) {
			eprintln!(
				"error while generating required programs after Game::switch_main call\n{err}"
			)
		}
	}
	pub fn regenerate_moves(&mut self, prev_id: WorldId) -> anyhow::Result<()> {
		self.moves = IngameWorld::generate(self, self.main_id)
			.with_context(|| "IngameWorld::generate failed in Game::regenerate_moves")?;
		self.generate_program_for(prev_id)
			.with_context(|| "generate_program_for failed in Game::regenerate_moves")?;
		Ok(())
	}
	/// recursively generates the programs needed for `wid` to function \
	/// (actually more than we need! this will recursively generate programs for every single world inside every single world etc inside wid)
	pub fn generate_programs_for(&mut self, wid: WorldId) -> anyhow::Result<()> {
		let w = self.worlds.at(wid).with_context(|| {
			format!("no world with id {wid}\nneeded to regenerate needed programs for {wid}")
		})?;

		let mut wids = w
			.find_foreigns()
			.map(|(coords, (wid, _, _))| wid)
			.collect::<Vec<_>>();
		wids.sort();
		wids.dedup();

		for child_wid in wids {
			self.generate_program_for(child_wid)
				.with_context(|| format!("error while updating programs for foreigns in {wid}"))?;
		}
		Ok(())
	}
	pub fn generate_program_for(&mut self, wid: WorldId) -> anyhow::Result<()> {
		let program = match processor::world_to_instructions(self, wid) {
			Ok(a) => Some(a),
			Err(err) => {
				eprintln!("failed to generate instructions for world {wid}\n{err}");
				None
			}
		};
		let prev_w = self.worlds.at(wid).with_context(|| {
			format!("there is no world {wid}, so the world we switched from doesn't exist")
		})?;

		self.programs.insert(
			wid,
			(program, prev_w.inputs_count(), prev_w.outputs_count()),
		);

		Ok(())
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
