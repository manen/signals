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
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.background.render(d, det, scale);
		self.foreground.render(d, det, scale);
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		if let Some(ret) = self.foreground.pass_event(event, det, scale) {
			Some(ret)
		} else {
			self.background.pass_event(event, det, scale)
		}
	}
}
