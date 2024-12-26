use crate::world::{Move, RenderedWorld, Signal, World};

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

#[derive(Clone, Debug, Default)]
pub struct IngameWorld {
	pub id: Option<usize>,
	pub moves: Vec<Move>,
	pub children: Vec<IngameWorld>,
}
impl IngameWorld {
	fn tick(&mut self, game: &mut Game, ret: impl FnMut(Move)) {
		let new_moves = game
			.world_mut(self.id)
			.tick(std::mem::take(&mut self.moves), |_, _, _| {});
		self.process_moves(new_moves, ret)
	}
	fn tick_children(&mut self, game: &mut Game) {
		for (i, child) in self.children.iter_mut().enumerate() {
			child.tick(game, |m| match m {
				Move::Output { id, signal } => self.moves.push(Move::Foreign {
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
		if self.children.len() - 1 <= inst_id {
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
	i: Option<usize>,

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
}
