use std::collections::HashMap;

use crate::{
	gfx,
	world::{world_coords_into_chunk_coords, Chunk, Move, World},
};

pub type Drawmap = HashMap<(i32, i32), Chunk<gfx::DrawType>>;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RenderedWorld {
	world: World,
	drawmap: Drawmap, // TODO move all world drawing shit to this file and implement sui::Layable if possible
}
impl AsRef<World> for RenderedWorld {
	fn as_ref(&self) -> &World {
		&self.world
	}
}
impl AsMut<World> for RenderedWorld {
	fn as_mut(&mut self) -> &mut World {
		&mut self.world
	}
}
impl RenderedWorld {
	pub fn new(world: World) -> Self {
		Self {
			world,
			drawmap: Default::default(),
		}
	}
	pub fn take(self) -> World {
		self.world
	}

	fn drawmap_reset(&mut self) {
		for (_, drawmap) in &mut self.drawmap {
			*drawmap = gfx::DRAWMAP_DEFAULT
		}
	}

	pub fn drawmap_at(&self, chunk_coords: (i32, i32)) -> &gfx::Drawmap {
		if let Some(original) = self.drawmap.get(&chunk_coords) {
			original
		} else {
			&gfx::DRAWMAP_DEFAULT
		}
	}

	pub fn tick(&mut self, moves: Vec<Move>) -> Vec<Move> {
		use std::mem::take;
		self.drawmap_reset();

		let mut drawmap = take(&mut self.drawmap);
		let moves = self.world.tick(moves, |x, y, dt| {
			drawtype_set_at(&mut drawmap, x, y, dt);
		});
		self.drawmap = drawmap;

		moves
	}
}

fn drawtype_set_at(drawmap: &mut Drawmap, x: i32, y: i32, dt: gfx::DrawType) {
	let (chunk_coords, (block_x, block_y)) = world_coords_into_chunk_coords(x, y);
	if let Some(drawmap) = drawmap.get_mut(&chunk_coords) {
		drawmap.map_at(block_x, block_y, |_| dt);
	} else {
		let mut def = gfx::DRAWMAP_DEFAULT;
		*(def
			.mut_at(block_x, block_y)
			.expect("world_coords_into_chunk_coords broke")) = dt;
		drawmap.insert(chunk_coords, def);
	}
}
