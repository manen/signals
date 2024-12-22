use crate::world::{RenderedWorld, World};

// how are we gonna get a whole world inside a block? i hear you asking
// create a block that contains: world id, input id, output id
// for now we can just hardcode for example signals from the left will be treated as input,
// while all other sides will output the output
// this is actually feasible but i'm soooooo tired gn

// very proof of concept-y
#[derive(Clone, Debug, Default)]
pub struct Game {
	// the way this works is when we switch to another world we write the index of
	// the world we switched to in i, when we switch back we read i to figure out worlds[?] is main
	pub main: RenderedWorld,
	worlds: Vec<World>,
	i: Option<usize>,
}
impl Game {
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
