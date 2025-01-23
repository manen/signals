use std::hash::Hash;

mod worlds;
pub use worlds::*;

mod ingameworld;
pub use ingameworld::*;

use crate::{gfx::DrawType, world::World};

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
}
impl Game {
	pub fn tick(&mut self) -> anyhow::Result<()> {
		// reset the drawmap
		for (_, c) in self.drawmap.chunks_mut() {
			*c = Default::default();
		}

		if let Some(world) = self.worlds.at_mut(self.main_id) {
			let new_main_moves = world.tick(std::mem::take(&mut self.moves.moves), |x, y, dt| {
				*self.drawmap.mut_at(x, y) = self
					.drawmap
					.at(x, y)
					.copied()
					.unwrap_or(DrawType::Off)
					.apply_new(dt);
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
		if let Err(err) = self.regenerate_moves() {
			eprintln!("error while regenerating moves after Game::switch_main call\n{err}")
		}
	}
	pub fn regenerate_moves(&mut self) -> anyhow::Result<()> {
		self.moves = IngameWorld::generate(self, self.main_id)?;
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
