use crate::Layable;

#[derive(Clone, Debug)]
/// renders the two components in the same place, overlapping each other
pub struct Overlay<A: Layable, B: Layable> {
	foreground: A,
	background: B,
}
impl<A: Layable, B: Layable> Overlay<A, B> {
	pub fn new(background: B, foreground: A) -> Self {
		Self {
			foreground,
			background,
		}
	}
}
impl<A: Layable, B: Layable> Layable for Overlay<A, B> {
	fn size(&self) -> (i32, i32) {
		let (a_w, a_h) = self.foreground.size();
		let (b_w, b_h) = self.background.size();

		(a_w.max(b_w), a_h.max(b_h))
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		let (w, h) = self.size();
		let det = crate::Details {
			aw: det.aw.min(w),
			ah: det.ah.min(h),
			..det
		};

		self.background.render(d, det, scale);
		self.foreground.render(d, det, scale);
	}
	fn pass_event(&self, event: crate::core::Event) -> Option<crate::core::Event> {
		if let Some(ret) = self.foreground.pass_event(event) {
			Some(ret)
		} else {
			self.background.pass_event(event)
		}
	}
}
