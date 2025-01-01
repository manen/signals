// this file contains structs that implement sui::Layable for everything ingame
use crate::{
	gfx,
	world::{self, World},
};
use sui::Layable;

const PREVIEW_SCALE: f32 = 1.0 / 4.0;

#[derive(Copy, Clone, Debug)]
pub struct WorldPreview<'a> {
	world: &'a World,
}
impl<'a> WorldPreview<'a> {
	pub fn new(world: &'a World) -> Self {
		Self { world }
	}
}
impl<'a> Layable for WorldPreview<'a> {
	fn size(&self) -> (i32, i32) {
		let (w_size, _) = self.world.size_and_offset();

		let total_size_per_chunk =
			world::CHUNK_SIZE as f32 * world::BLOCK_SIZE as f32 * PREVIEW_SCALE;
		let total_size_per_chunk = total_size_per_chunk as i32;

		(
			w_size.0 * total_size_per_chunk,
			w_size.1 * total_size_per_chunk,
		)
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: sui::Details, scale: f32) {
		let (_, w_offset) = self.world.size_and_offset();

		gfx::render_basic_world(
			&self.world,
			d,
			gfx::PosInfo {
				base: (
					det.x
						+ (w_offset.0 as f32
							* world::CHUNK_SIZE as f32
							* world::BLOCK_SIZE as f32
							* PREVIEW_SCALE * scale) as i32,
					det.y
						+ (w_offset.1 as f32
							* world::CHUNK_SIZE as f32
							* world::BLOCK_SIZE as f32
							* PREVIEW_SCALE * scale) as i32,
				),
				scale: scale * PREVIEW_SCALE,
			},
		);
	}
	fn pass_event(&self, _: sui::core::Event) -> Option<sui::core::Event> {
		None
	}
}
