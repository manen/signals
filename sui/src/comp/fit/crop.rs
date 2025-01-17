use crate::Layable;

#[derive(Clone, Debug)]
pub struct Crop<L: Layable> {
	layable: L,
}
impl<L: Layable> Crop<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}
}
impl<L: Layable> Layable for Crop<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		unsafe {
			raylib::ffi::BeginScissorMode(
				det.x,
				det.y,
				(det.aw as f32 * scale) as i32,
				(det.ah as f32 * scale) as i32,
			)
		};
		self.layable.render(d, det, scale);
		unsafe { raylib::ffi::EndScissorMode() };
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		self.layable.pass_event(event, det, scale)
	}
}
