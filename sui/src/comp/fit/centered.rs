use crate::Layable;

#[derive(Clone, Debug)]
/// self.size() is self.layable.size(), the centering only happens on self.render()
pub struct Centered<L: Layable> {
	layable: L,
}
impl<L: Layable> Centered<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}
}
impl<L: Layable> Layable for Centered<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		let (l_w, l_h) = self.layable.size();

		let (base_x, base_y) = (
			det.x + (det.aw as f32 / 2.0 - l_w as f32 / 2.0 * scale) as i32,
			det.y + (det.ah as f32 / 2.0 - l_h as f32 / 2.0 * scale) as i32,
		);
		let l_det = crate::Details {
			x: base_x,
			y: base_y,
			..det
		};

		self.layable.render(d, l_det, scale);
	}
	fn pass_event(&self, event: crate::core::Event) -> Option<crate::core::Event> {
		self.layable.pass_event(event)
	}
}
